use std::error::Error;
use std::fmt::Display;
use std::path::Path;
use std::sync::LazyLock;
use std::{fmt, fs};

use crate::ast::*;
use crate::types::Type;
use miette::{Diagnostic, IntoDiagnostic, NamedSource, Result, SourceSpan, miette};
use pest::iterators::{Pair, Pairs};
use pest::pratt_parser::{Assoc, Op, PrattParser};
use pest::{Parser, Position, RuleType, Span};

use pest::error::Error as PestParseError;
use pest::error::ErrorVariant as PestParseErrorVariant;

#[derive(pest_derive::Parser)]
#[grammar = "grammar.pest"] // relative to src
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
    fn from_span<'a>(message: String, span: Span<'a>) -> Self {
        let span_start = span.start();
        let span_end = span.end();
        let len = span_end - span_start;
        let snippet = SourceSpan::new(span_start.into(), len);

        ParseError {
            err_message: message,
            snippet,
            help: None,
        }
    }

    fn enumerate<F, R>(rules: &[R], f: &mut F) -> String
    where
        F: FnMut(&R) -> String,
    {
        match rules.len() {
            1 => f(&rules[0]),
            2 => format!("{} or {}", f(&rules[0]), f(&rules[1])),
            l => {
                let non_separated = f(&rules[l - 1]);
                let separated = rules
                    .iter()
                    .take(l - 1)
                    .map(f)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}, or {}", separated, non_separated)
            }
        }
    }

    fn from_expected<'a, R: RuleType>(
        message: String,
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

    fn from_pest_message<R: RuleType>(err: PestParseError<R>, err_message: String) -> ParseError {
        let help = Some(err.variant.message().to_string());

        let span = match err.location {
            pest::error::InputLocation::Pos(pos) => (pos, pos + 1),
            pest::error::InputLocation::Span((start, end)) => (start, end),
        };

        let snippet = SourceSpan::new(span.0.into(), span.1 - span.0);

        ParseError {
            err_message,
            snippet,
            help,
        }
    }

    fn from_pest<R: RuleType>(err: PestParseError<R>) -> ParseError {
        let message = "Grammar parsing error".to_string();
        Self::from_pest_message(err, message)
    }
}

pub fn parse_file<'a>(source: &'a str, file_path: &'a Path) -> Result<File<'a>> {
    let mut res = File::new(source, file_path);

    let pairs = AssemblyParser::parse(Rule::Program, source).map_err(ParseError::from_pest)?;

    let program = res.statements_mut();

    for pair in pairs.into_iter() {
        match pair.as_rule() {
            Rule::Statement => {
                // println!("found statement \"{}\"", pair);
                program.push(parse_statement(pair)?);
            }
            Rule::EOI => (),
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
        r => Err(ParseError::from_span(
            format!("Unexpected statement rule: {:?}", r),
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
        r => Err(ParseError::from_span(
            format!("Unexpected label rule: {:?}", r),
            inner.as_span(),
        ))?,
    }
}

fn parse_block_label(pair: Pair<Rule>) -> Result<AstNode<Statement>> {
    let mut name: Result<String> =
        Err(ParseError::from_span("Block label name not found".to_string(), pair.as_span()).into());
    let mut body: Result<Vec<AstNode<Statement>>> =
        Err(ParseError::from_span("Block label body not found".to_string(), pair.as_span()).into());

    let span = pair.as_span();

    for item in pair.into_inner() {
        match item.as_rule() {
            Rule::LabelIdentifier => name = Ok(item.to_string()),
            Rule::Block => body = Ok(parse_block(item)?),
            r => Err(ParseError::from_expected(
                "Block label parsing error".to_string(),
                vec![Rule::LabelIdentifier, Rule::Block],
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
            r => Err(ParseError::from_span(
                format!("Unexpected block rule: {:?}", r),
                item.as_span(),
            ))?,
        }
    }
    Ok(stmts)
}

fn parse_instruction(pair: Pair<Rule>) -> Result<AstNode<Statement>> {
    let mut name = String::new();
    let mut params = Vec::new();
    let span = pair.as_span();

    for item in pair.into_inner() {
        // println!("istr item: {:#?}", item);
        match item.as_rule() {
            Rule::InstructionLabel => name = item.to_string(),
            Rule::InstructionParameters => params = parse_instruction_parameters(item)?,
            r => return Err(miette!("Unexpected instruction rule: {:?}", r)),
        };
    }

    Ok(AstNode::new(
        Statement::Instruction(AstInstruction::new(name, params)),
        span,
    ))
}

fn parse_instruction_parameters(pair: Pair<Rule>) -> Result<Vec<AstNode<TypedExpr>>> {
    let mut params = Vec::new();

    for item in pair.into_inner() {
        // println!("istr parameter item: {:#?}", item);
        match item.as_rule() {
            Rule::Expr => params.push(parse_expr(item.into_inner())?),
            r => return Err(miette!("Unexpected itsr parameter rule: {:?}", r)),
        };
    }

    Ok(params)
}

fn parse_expr(pairs: Pairs<Rule>) -> Result<AstNode<TypedExpr>> {
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::Literal => parse_literal(primary),
            Rule::Identifier => Ok(AstNode::from_pair(
                TypedExpr::unknown(Expr::Identity(primary.as_str().to_string())),
                primary,
            )),
            Rule::Expr => parse_expr(primary.into_inner()),
            r => Err(miette!("Unexpected primary: {:?}", r)),
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
                _ => return Err(miette!("Unexpected infix op: {:?}", op)),
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
                    "Ast parsing error".to_string(),
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
        .parse(pairs)
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
        r => Err(miette!("Unexpected literal rule: {:?}", r)),
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
        r => Err(miette!("Unexpected hexadecimal rule: {:?}", r)),
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
