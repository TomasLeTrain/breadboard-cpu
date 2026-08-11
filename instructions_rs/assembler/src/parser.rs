use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::path::Path;
use std::sync::LazyLock;

use crate::ast::*;
use crate::types::Type;
use miette::{Diagnostic, Result, SourceSpan};
use pest::iterators::Pair;
use pest::pratt_parser::{Assoc, Op, PrattParser};
use pest::{Parser, RuleType, Span};

use pest::error::Error as PestParseError;
use pest::error::ErrorVariant as PestParseErrorVariant;

#[derive(pest_derive::Parser)]
#[grammar = "grammar.pest"]
pub struct AssemblyParser;

pub static PRATT_PARSER: LazyLock<PrattParser<Rule>> = LazyLock::new(|| {
    use Assoc::*;
    use Rule::*;

    PrattParser::new()
        // Lowest precedence first
        .op(Op::infix(logical_or, Left)) // ||
        .op(Op::infix(logical_and, Left)) // &&
        .op(Op::infix(eq, Left) | Op::infix(ne, Left)) // == !=
        .op(Op::infix(le, Left) | Op::infix(ge, Left) | Op::infix(lt, Left) | Op::infix(gt, Left)) // <= >= < >
        .op(Op::infix(bit_or, Left)) // |
        .op(Op::infix(bit_xor, Left)) // ^
        .op(Op::infix(bit_and, Left)) // &
        .op(Op::infix(shift_left, Left) | Op::infix(shift_right, Left)) // << >>
        .op(Op::infix(add, Left) | Op::infix(subtract, Left)) // + -
        .op(Op::infix(multiply, Left) | Op::infix(divide, Left) | Op::infix(modulo, Left)) // * / %
        .op(Op::infix(power, Right)) // ** (right-associative)
        // Highest precedence
        .op(Op::prefix(logical_not) | Op::prefix(negation) | Op::prefix(bit_negation)) // ! - ~ (unary)
});

/// parse a file
pub fn parse_file<'a>(source: &'a str, file_path: &'a Path) -> Result<FileAst<'a>> {
    let mut res = FileAst::new(source, file_path);

    let pairs = AssemblyParser::parse(Rule::Program, source).map_err(ParseError::from_pest)?;
    // println!("{pairs:#?}");

    let program = res.statements_mut();

    for pair in pairs.into_iter() {
        match pair.as_rule() {
            Rule::Statement => {
                program.push(parse_statement(pair)?);
            }
            Rule::COMMENT | Rule::EOI => (),
            _ => {}
        }
    }

    Ok(res)
}

fn parse_statement(pair: Pair<Rule>) -> Result<AstNode<Statement>> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::InstructionStatement => parse_instruction(inner),
        Rule::LabelStatement => parse_label(inner),
        Rule::BlockLabel => parse_block_label(inner),
        r => Err(ParseError::from_expected(
            "Statement parsing error".to_string(),
            vec![
                Rule::InstructionStatement,
                Rule::LabelStatement,
                Rule::BlockLabel,
            ],
            vec![r],
            inner.as_span(),
        ))?,
    }
}

fn parse_label(pair: Pair<Rule>) -> Result<AstNode<Statement>> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::LabelIdentifier => Ok(AstNode::from_pair(
            Statement::Label {
                name: inner.to_string(),
            },
            inner,
        )),
        r => Err(ParseError::from_expected(
            "Label parsing error".to_string(),
            vec![Rule::LabelIdentifier],
            vec![r],
            inner.as_span(),
        ))?,
    }
}

fn parse_block_label(pair: Pair<Rule>) -> Result<AstNode<Statement>> {
    let mut name: Result<String> =
        Err(ParseError::from_span("Block label name not found", pair.as_span()).into());
    let mut body: Result<Vec<AstNode<Statement>>> =
        Err(ParseError::from_span("Block label body not found", pair.as_span()).into());

    let span = pair.as_span();

    for item in pair.into_inner() {
        match item.as_rule() {
            Rule::LabelIdentifier => name = Ok(item.to_string()),
            Rule::Block => body = Ok(parse_block(item)?),
            Rule::COMMENT => (),
            r => Err(ParseError::from_expected(
                "Block label parsing error",
                vec![Rule::LabelIdentifier, Rule::Block, Rule::COMMENT],
                vec![r],
                span,
            ))?,
        }
    }

    Ok(AstNode::new(
        Statement::BlockLabel {
            name: name?,
            body: body?,
        },
        span,
    ))
}

fn parse_block(pair: Pair<Rule>) -> Result<Vec<AstNode<Statement>>> {
    let mut stmts = Vec::new();
    for item in pair.into_inner() {
        match item.as_rule() {
            Rule::Statement => stmts.push(parse_statement(item)?),
            Rule::COMMENT => (),
            r => Err(ParseError::from_expected(
                "Block parsing error".to_string(),
                vec![Rule::Statement, Rule::COMMENT],
                vec![r],
                item.as_span(),
            ))?,
        }
    }
    Ok(stmts)
}

fn parse_instruction(pair: Pair<Rule>) -> Result<AstNode<Statement>> {
    // let mut name: Result<String> =
    //     Err(ParseError::from_span("Block label name not found".to_string(), pair.as_span()).into());
    // let mut body: Result<Vec<AstNode<Statement>>> =
    //     Err(ParseError::from_span("Block label body not found".to_string(), pair.as_span()).into());

    let mut name: Result<String> =
        Err(ParseError::from_span("Instruction name not found".to_string(), pair.as_span()).into());

    let mut params: Vec<AstNode<'_, TypedExpr<'_>>> = Vec::new();
    let og_span = pair.as_span();

    let mut span = Span::new(
        pair.get_input(),
        pair.as_span().start(),
        pair.as_span().start() + 1,
    )
    .unwrap();

    // used to merge spans, even if non contiguous
    let mut merge = |other: Span| -> () {
        span = Span::new(
            span.get_input(),
            core::cmp::min(span.start(), other.start()),
            core::cmp::max(span.end(), other.end()),
        )
        .unwrap();
    };

    for item in pair.into_inner() {
        match item.as_rule() {
            Rule::InstructionLabel => {
                // merge span covering label in case no params
                merge(item.as_span());
                name = Ok(item.to_string());
            }
            Rule::InstructionParameters => params = parse_instruction_parameters(item)?,
            Rule::COMMENT => (),
            r => Err(ParseError::from_expected(
                "Instruction parsing error".to_string(),
                vec![
                    Rule::InstructionLabel,
                    Rule::InstructionParameters,
                    Rule::COMMENT,
                ],
                vec![r],
                item.as_span(),
            ))?,
        };
    }

    // merge span to the last param, if exists
    if let Some(param) = params.iter().last() {
        merge(param.span());
    }

    Ok(AstNode::new(
        Statement::Instruction(AstInstruction::new(name?, params)),
        span,
    ))
}

fn parse_instruction_parameters(pair: Pair<Rule>) -> Result<Vec<AstNode<TypedExpr>>> {
    let mut params = Vec::new();

    for item in pair.into_inner() {
        match item.as_rule() {
            Rule::Expr => params.push(parse_expr(item.into_inner())?),
            Rule::COMMENT => (),
            r => Err(ParseError::from_expected(
                "Instruction parameter parsing error".to_string(),
                vec![Rule::Expr, Rule::COMMENT],
                vec![r],
                item.as_span(),
            ))?,
        };
    }

    Ok(params)
}

fn parse_expr<'a>(
    pairs: impl Iterator<Item = pest::iterators::Pair<'a, Rule>>,
) -> Result<AstNode<'a, TypedExpr<'a>>> {
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::Literal => parse_literal(primary),
            Rule::Identifier => Ok(AstNode::from_pair(
                TypedExpr::unknown(Expr::Identity(primary.as_str().to_string())),
                primary,
            )),
            Rule::Expr => {
                let inner = primary.into_inner();

                // https://github.com/pest-parser/pest/discussions/1131
                let no_comments: Vec<pest::iterators::Pair<'a, Rule>> = inner
                    .flatten()
                    .filter(|x| x.as_rule() != Rule::COMMENT)
                    .collect();

                parse_expr(no_comments.into_iter())
            }
            r => Err(ParseError::from_expected(
                "Primary parsing error".to_string(),
                vec![Rule::Literal, Rule::Identifier, Rule::Expr],
                vec![r],
                primary.as_span(),
            ))?,
        })
        .map_infix(|lhs, op, rhs| {
            // Handle binary operations
            let bin_op = match op.as_rule() {
                Rule::add => BinaryOp::Add,
                Rule::subtract => BinaryOp::Sub,
                Rule::multiply => BinaryOp::Mul,
                Rule::divide => BinaryOp::Div,
                Rule::modulo => BinaryOp::Mod,
                Rule::power => BinaryOp::Pow,

                Rule::shift_left => BinaryOp::ShiftLeft,
                Rule::shift_right => BinaryOp::ShiftRight,
                Rule::bit_and => BinaryOp::BitAnd,
                Rule::bit_xor => BinaryOp::BitXor,
                Rule::bit_or => BinaryOp::BitOr,

                Rule::eq => BinaryOp::Eq,
                Rule::ne => BinaryOp::Ne,
                Rule::le => BinaryOp::Le,
                Rule::ge => BinaryOp::Ge,
                Rule::lt => BinaryOp::Lt,
                Rule::gt => BinaryOp::Gt,

                Rule::logical_and => BinaryOp::And,
                Rule::logical_or => BinaryOp::Or,

                r => Err(ParseError::from_expected(
                    "Binary op parsing error".to_string(),
                    vec![
                        Rule::add,
                        Rule::subtract,
                        Rule::multiply,
                        Rule::divide,
                        Rule::modulo,
                        Rule::power,
                        Rule::shift_left,
                        Rule::shift_right,
                        Rule::bit_and,
                        Rule::bit_xor,
                        Rule::bit_or,
                        Rule::eq,
                        Rule::ne,
                        Rule::le,
                        Rule::ge,
                        Rule::lt,
                        Rule::gt,
                        Rule::logical_and,
                        Rule::logical_or,
                    ],
                    vec![r],
                    op.as_span(),
                ))?,
            };
            Ok(AstNode::from_pair(
                TypedExpr::unknown(Expr::Binary {
                    op: bin_op,
                    left: Box::new(lhs?),
                    right: Box::new(rhs?),
                }),
                op,
            ))
        })
        .map_prefix(|op, rhs| {
            // Handle unary operations
            let un_op = match op.as_rule() {
                Rule::subtract => UnaryOp::Neg,
                Rule::logical_not => UnaryOp::Not,
                Rule::bit_negation => UnaryOp::BitNegation,
                r => Err(ParseError::from_expected(
                    "Prefix parsing error".to_string(),
                    vec![Rule::subtract, Rule::logical_not, Rule::bit_negation],
                    vec![r],
                    op.as_span(),
                ))?,
            };

            Ok(AstNode::from_pair(
                TypedExpr::unknown(Expr::Unary {
                    op: un_op,
                    expr: Box::new(rhs?),
                }),
                op,
            ))
        })
        .parse(pairs.filter(|x| x.as_rule() != Rule::COMMENT))
}

fn parse_literal(pair: Pair<Rule>) -> Result<AstNode<TypedExpr>> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::Int => Ok(AstNode::from_pair(
            TypedExpr::new(Expr::Int(inner.as_str().parse().unwrap()), Type::Int),
            inner,
        )),
        Rule::Hexadecimal => parse_hexadecimal(inner),
        Rule::Bool => Ok(AstNode::from_pair(
            TypedExpr::new(Expr::Bool(inner.as_str() == "true"), Type::Bool),
            inner,
        )),
        Rule::String => {
            let s = inner.as_str();
            // removes "" quotes
            Ok(AstNode::from_pair(
                TypedExpr::new(Expr::String(s[1..s.len() - 1].to_string()), Type::String),
                inner,
            ))
        }
        Rule::Character => {
            let s = inner.as_str();
            let c = unescape_char(&s[1..s.len() - 1]);

            Ok(AstNode::from_pair(
                TypedExpr::new(Expr::Char(c as u8), Type::Character),
                inner,
            ))
        }
        r => Err(ParseError::from_expected(
            "Literal parsing error".to_string(),
            vec![
                Rule::Int,
                Rule::Hexadecimal,
                Rule::Bool,
                Rule::String,
                Rule::Character,
            ],
            vec![r],
            inner.as_span(),
        ))?,
    }
}

fn parse_hexadecimal(pair: Pair<Rule>) -> Result<AstNode<TypedExpr>> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::Int => Ok(AstNode::from_pair(
            TypedExpr::new(
                Expr::Int(i64::from_str_radix(inner.as_str(), 16).unwrap()),
                Type::Int,
            ),
            inner,
        )),
        r => Err(ParseError::from_expected(
            "Hexadecimal parsing error".to_string(),
            vec![Rule::Int],
            vec![r],
            inner.as_span(),
        ))?,
    }
}

/// Turns string of length <= 2 into corresponding character.
/// Turns literal escape sequences like "\\n" into the actual character '\n'.
fn unescape_char(string: &str) -> char {
    assert!(string.chars().count() <= 2);

    let mut iter = string.chars();

    let first_char = iter.next().unwrap();

    if first_char == '\\' {
        let second_char = iter.next().unwrap();
        match second_char {
            't' => '\t',
            'r' => '\r',
            'n' => '\n',
            '\'' => '\'',
            '"' => '"',
            '\\' => '\\',
            c => c,
        }
    } else {
        first_char
    }
}

// TODO: make error code generic to distinguish grammar parsing vs. ast parsing

#[derive(Debug, Diagnostic)]
#[diagnostic(code(assembler::parse_error))]
pub struct ParseError {
    #[label]
    pub snippet: SourceSpan,
    pub err_message: String,
    #[help]
    pub help: Option<String>,
}

impl Error for ParseError {}

impl Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.err_message)
    }
}

impl ParseError {
    pub fn from_span<'a>(message: impl Into<String>, span: Span<'a>) -> Self {
        let span_start = span.start();
        let span_end = span.end();
        let len = span_end - span_start;
        let snippet = SourceSpan::new(span_start.into(), len);

        ParseError {
            err_message: message.into(),
            snippet,
            help: None,
        }
    }

    /// creates error message with a help message detailing expected/unexpected rules
    pub fn from_expected<'a, R: RuleType>(
        message: impl Into<String>,
        expected: Vec<R>,
        unexpected: Vec<R>,
        span: Span<'a>,
    ) -> Self {
        Self::from_pest_message(
            PestParseError::new_from_span(
                PestParseErrorVariant::ParsingError {
                    positives: expected,
                    negatives: unexpected,
                },
                span,
            ),
            message,
        )
    }

    fn from_pest_message<R: RuleType>(err: PestParseError<R>, err_message: impl Into<String>) -> ParseError {
        let help = Some(err.variant.message().to_string());

        let span = match err.location {
            pest::error::InputLocation::Pos(pos) => (pos, pos + 1),
            pest::error::InputLocation::Span((start, end)) => (start, end),
        };

        let snippet = SourceSpan::new(span.0.into(), span.1 - span.0);

        ParseError {
            err_message: err_message.into(),
            snippet,
            help,
        }
    }

    fn from_pest<R: RuleType>(err: PestParseError<R>) -> ParseError {
        let message = "Grammar parsing error".to_string();
        Self::from_pest_message(err, message)
    }
}
