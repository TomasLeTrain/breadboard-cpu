use std::sync::{Arc, LazyLock};

use crate::ast::*;
use crate::error::ParseError;
use crate::eval::ExprValue;
use crate::types::Type;

use miette::Result;
use pest::Parser;
use pest::iterators::Pair;
use pest::pratt_parser::{Assoc, Op, PrattParser};

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
        .op(Op::prefix(logical_not) | Op::prefix(negation) | Op::prefix(bit_negation))
    // ! - ~ (unary)
});

/// parse a file
pub fn parse_file(source: Source) -> Result<Ast> {
    let mut program = Ast::new();

    let pairs = AssemblyParser::parse(Rule::Program, source.source())
        .map_err(|e| ParseError::from_pest(e, &source))?;
    println!("bruh: {:#?}", pairs);

    for pair in pairs.into_iter() {
        match pair.as_rule() {
            Rule::Statement => {
                if let Some(statement) = parse_statement(pair, &source)? {
                    program.push(statement);
                }
            }
            Rule::COMMENT | Rule::EOI => (),
            _ => {}
        }
    }

    Ok(program)
}

fn parse_statement(pair: Pair<Rule>, source: &Source) -> Result<Option<StatementNode>> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::FunctionStatement => Ok(Some(parse_function(inner, source)?)),
        Rule::FunctionCall => Ok(Some(parse_function_call_statement(inner, source)?)),
        Rule::InstructionStatement => Ok(Some(parse_instruction(inner, source)?)),
        Rule::ReturnStatement => Ok(Some(parse_return_statement(inner, source)?)),
        Rule::LabelStatement => Ok(Some(parse_label(inner, source)?)),
        Rule::BlockLabel => Ok(Some(parse_block_label(inner, source)?)),
        Rule::Block => Ok(Some(parse_block_statement(inner, source)?)),
        Rule::COMMENT => Ok(None),
        r => Err(ParseError::from_expected(
            "Statement parsing error".to_string(),
            vec![
                Rule::InstructionStatement,
                Rule::LabelStatement,
                Rule::BlockLabel,
                Rule::COMMENT,
            ],
            vec![r],
            &AstSpan::from_span(inner.as_span(), source),
        ))?,
    }
}

fn parse_return_statement(pair: Pair<Rule>, source: &Source) -> Result<StatementNode> {
    let span = AstSpan::from_span(pair.as_span(), source);
    let inner = pair.into_inner().next().unwrap();

    let return_kind = match inner.as_rule() {
        Rule::Block => ReturnKind::Block(parse_block(inner, source)?),
        Rule::Expr => ReturnKind::Expr(parse_expr(inner.into_inner(), source)?),
        _ => unreachable!(),
    };

    Ok(AstNode::new(
        Statement::new(StatementKind::Return(return_kind)),
        span,
    ))
}

fn parse_block_statement(pair: Pair<Rule>, source: &Source) -> Result<StatementNode> {
    let span = AstSpan::from_span(pair.as_span(), source);
    let body = parse_block(pair, source)?;

    Ok(AstNode::new(
        Statement::new(StatementKind::Block { body }),
        span,
    ))
}

fn parse_function_call_statement(pair: Pair<Rule>, source: &Source) -> Result<StatementNode> {
    let span = AstSpan::from_span(pair.as_span(), source);
    Ok(AstNode::new(
        Statement::new(StatementKind::FunctionCall(parse_function_call(
            pair, source,
        )?)),
        span,
    ))
}

fn parse_function_call(pair: Pair<Rule>, source: &Source) -> Result<FunctionCall> {
    let mut name: Result<String> = Err(ParseError::from_span(
        "Function call name not found".to_string(),
        &AstSpan::from_span(pair.as_span(), source),
    )
    .into());

    let mut params: Vec<AstNode<Expr>> = Vec::new();

    for item in pair.into_inner() {
        match item.as_rule() {
            Rule::FunctionCallName => {
                // merge span covering label in case no params
                name = Ok(item.to_string());
            }
            Rule::ExprParameters => {
                params = parse_expr_parameters(item, source)?;
            }
            Rule::COMMENT => (),
            r => Err(ParseError::from_expected(
                "Function call parsing error".to_string(),
                vec![Rule::FunctionCallName, Rule::ExprParameters, Rule::COMMENT],
                vec![r],
                &AstSpan::from_span(item.as_span(), source),
            ))?,
        };
    }

    Ok(FunctionCall::new(name?, params))
}

fn parse_function(pair: Pair<Rule>, source: &Source) -> Result<StatementNode> {
    let mut name: Result<String> = Err(ParseError::from_span(
        "Instruction name not found".to_string(),
        &AstSpan::from_span(pair.as_span(), source),
    )
    .into());

    let mut params: Vec<AstNode<TypedParameter>> = Vec::new();

    let mut block: Result<Vec<StatementNode>> = Err(ParseError::from_span(
        "Function Block not found".to_string(),
        &AstSpan::from_span(pair.as_span(), source),
    )
    .into());

    let mut span = AstSpan::new(
        pair.as_span().start(),
        pair.as_span().start() + 1,
        Arc::clone(source),
    );

    // used to merge spans, even if non contiguous
    let mut merge = |start: usize, end: usize| -> () {
        span.set_span(
            core::cmp::min(span.start(), start),
            core::cmp::max(span.end(), end),
        );
    };

    for item in pair.into_inner() {
        match item.as_rule() {
            Rule::FunctionName => {
                // merge span covering label in case no params
                merge(item.as_span().start(), item.as_span().end());
                name = Ok(item.to_string());
            }
            Rule::FunctionParameters => {
                merge(item.as_span().start(), item.as_span().end());
                params = parse_function_parameters(item, source)?;
            }
            Rule::Block => {
                merge(item.as_span().start(), item.as_span().end());
                block = Ok(parse_block(item, source)?);
            }
            Rule::COMMENT => (),
            r => Err(ParseError::from_expected(
                "Function parsing error".to_string(),
                vec![
                    Rule::FunctionName,
                    Rule::FunctionParameters,
                    Rule::Block,
                    Rule::COMMENT,
                ],
                vec![r],
                &AstSpan::from_span(item.as_span(), source),
            ))?,
        };
    }

    Ok(AstNode::new(
        Statement::new(StatementKind::Function(Function::new(
            name?, params, block?,
        ))),
        span,
    ))
}
fn parse_function_parameters(
    pair: Pair<Rule>,
    source: &Source,
) -> Result<Vec<AstNode<TypedParameter>>> {
    let mut params = Vec::new();

    for item in pair.into_inner() {
        match item.as_rule() {
            Rule::TypedParameter => params.push(parse_typed_param(item, source)?),
            Rule::COMMENT => (),
            r => Err(ParseError::from_expected(
                "Function parameters parsing error".to_string(),
                vec![Rule::Expr, Rule::COMMENT],
                vec![r],
                &AstSpan::from_span(item.as_span(), source),
            ))?,
        };
    }

    Ok(params)
}

fn parse_typed_param(pair: Pair<Rule>, source: &Source) -> Result<AstNode<TypedParameter>> {
    let mut name: Result<String> = Err(ParseError::from_span(
        "Parameter name not found".to_string(),
        &AstSpan::from_span(pair.as_span(), source),
    )
    .into());

    let mut ty: Result<Type> = Err(ParseError::from_span(
        "Parameter type not found".to_string(),
        &AstSpan::from_span(pair.as_span(), source),
    )
    .into());

    let span = AstSpan::from_span(pair.as_span(), source);

    for item in pair.into_inner() {
        match item.as_rule() {
            Rule::Identifier => {
                // merge span covering label in case no params
                name = Ok(item.to_string());
            }
            Rule::Type => {
                ty = Ok(parse_type(item, source)?);
            }
            Rule::COMMENT => (),
            r => Err(ParseError::from_expected(
                "Function Param parsing error".to_string(),
                vec![Rule::Identifier, Rule::Type, Rule::COMMENT],
                vec![r],
                &AstSpan::from_span(item.as_span(), source),
            ))?,
        };
    }

    Ok(AstNode::new(TypedParameter::new(name?, ty?), span))
}

fn parse_type(pair: Pair<Rule>, source: &Source) -> Result<Type> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::U16Type => Ok(Type::Int),
        Rule::U8Type => Ok(Type::Int),
        Rule::I16Type => Ok(Type::Int),
        Rule::I8Type => Ok(Type::Int),
        Rule::IntType => Ok(Type::Int),
        Rule::BoolType => Ok(Type::Bool),
        Rule::LabelType => Ok(Type::Label),
        Rule::AddrType => Ok(Type::Addr),
        r => Err(ParseError::from_expected(
            "Type parsing error".to_string(),
            vec![
                Rule::U16Type,
                Rule::U8Type,
                Rule::I16Type,
                Rule::I8Type,
                Rule::IntType,
                Rule::BoolType,
                Rule::LabelType,
                Rule::AddrType,
            ],
            vec![r],
            &AstSpan::from_span(inner.as_span(), source),
        ))?,
    }
}

fn parse_label(pair: Pair<Rule>, source: &Source) -> Result<StatementNode> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::LabelIdentifier => Ok(AstNode::from_pair(
            Statement::new(StatementKind::Label {
                name: inner.to_string(),
            }),
            inner,
            source,
        )),
        r => Err(ParseError::from_expected(
            "Label parsing error".to_string(),
            vec![Rule::LabelIdentifier],
            vec![r],
            &AstSpan::from_span(inner.as_span(), source),
        ))?,
    }
}

fn parse_block_label(pair: Pair<Rule>, source: &Source) -> Result<StatementNode> {
    let mut name: Result<String> = Err(ParseError::from_span(
        "Block label name not found",
        &AstSpan::from_span(pair.as_span(), source),
    )
    .into());
    let mut body: Result<Vec<StatementNode>> = Err(ParseError::from_span(
        "Block label body not found",
        &AstSpan::from_span(pair.as_span(), source),
    )
    .into());

    let span = AstSpan::from_span(pair.as_span(), source);

    for item in pair.into_inner() {
        match item.as_rule() {
            Rule::LabelIdentifier => name = Ok(item.to_string()),
            Rule::Block => body = Ok(parse_block(item, source)?),
            Rule::COMMENT => (),
            r => Err(ParseError::from_expected(
                "Block label parsing error",
                vec![Rule::LabelIdentifier, Rule::Block, Rule::COMMENT],
                vec![r],
                &span,
            ))?,
        }
    }

    Ok(AstNode::new(
        Statement::new(StatementKind::BlockLabel {
            name: name?,
            body: body?,
        }),
        span,
    ))
}

fn parse_block(pair: Pair<Rule>, source: &Source) -> Result<Vec<StatementNode>> {
    let mut statements = Vec::new();
    for item in pair.into_inner() {
        match item.as_rule() {
            Rule::Statement => {
                if let Some(statement) = parse_statement(item, source)? {
                    statements.push(statement);
                }
            }
            Rule::COMMENT => (),
            r => Err(ParseError::from_expected(
                "Block parsing error".to_string(),
                vec![Rule::Statement, Rule::COMMENT],
                vec![r],
                &AstSpan::from_span(item.as_span(), source),
            ))?,
        }
    }

    Ok(statements)
}

fn parse_instruction(pair: Pair<Rule>, source: &Source) -> Result<StatementNode> {
    let mut name: Result<String> = Err(ParseError::from_span(
        "Instruction name not found".to_string(),
        &AstSpan::from_span(pair.as_span(), source),
    )
    .into());

    let mut params: Vec<AstNode<Expr>> = Vec::new();

    let mut span = AstSpan::new(
        pair.as_span().start(),
        pair.as_span().start() + 1,
        Arc::clone(source),
    );

    // used to merge spans, even if non contiguous
    let mut merge = |start: usize, end: usize| -> () {
        span.set_span(
            core::cmp::min(span.start(), start),
            core::cmp::max(span.end(), end),
        );
    };

    for item in pair.into_inner() {
        match item.as_rule() {
            Rule::InstructionLabel => {
                // merge span covering label in case no params
                merge(item.as_span().start(), item.as_span().end());
                name = Ok(item.to_string());
            }
            Rule::ExprParameters => {
                merge(item.as_span().start(), item.as_span().end());
                params = parse_expr_parameters(item, source)?;
            }
            Rule::COMMENT => (),
            r => Err(ParseError::from_expected(
                "Instruction parsing error".to_string(),
                vec![Rule::InstructionLabel, Rule::ExprParameters, Rule::COMMENT],
                vec![r],
                &AstSpan::from_span(item.as_span(), source),
            ))?,
        };
    }

    Ok(AstNode::new(
        Statement::new(StatementKind::Instruction(AstInstruction::new(
            name?, params,
        ))),
        span,
    ))
}

fn parse_expr_parameters(pair: Pair<Rule>, source: &Source) -> Result<Vec<AstNode<Expr>>> {
    let mut params = Vec::new();

    for item in pair.into_inner() {
        match item.as_rule() {
            Rule::Expr => params.push(parse_expr(item.into_inner(), source)?),
            Rule::COMMENT => (),
            r => Err(ParseError::from_expected(
                "Instruction parameter parsing error".to_string(),
                vec![Rule::Expr, Rule::COMMENT],
                vec![r],
                &AstSpan::from_span(item.as_span(), source),
            ))?,
        };
    }

    Ok(params)
}

fn parse_function_call_expr(pair: Pair<Rule>, source: &Source) -> Result<AstNode<Expr>> {
    let span = AstSpan::from_span(pair.as_span(), source);

    Ok(AstNode::new(
        Expr::unknown(ExprKind::FunctionCall(parse_function_call(pair, source)?)),
        span,
    ))
}

fn parse_expr<'a>(
    pairs: impl Iterator<Item = pest::iterators::Pair<'a, Rule>>,
    source: &Source,
) -> Result<AstNode<Expr>> {
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::Literal => parse_literal(primary, source),
            Rule::FunctionCall => parse_function_call_expr(primary, source),
            Rule::Identifier => Ok(AstNode::from_pair(
                Expr::unknown(ExprKind::Identity(primary.as_str().to_string())),
                primary,
                source,
            )),
            Rule::Expr => {
                let inner = primary.into_inner();

                // https://github.com/pest-parser/pest/discussions/1131
                let no_comments: Vec<pest::iterators::Pair<'a, Rule>> = inner
                    .filter(|x| !matches!(x.as_rule(), Rule::COMMENT))
                    .collect();

                parse_expr(no_comments.into_iter(), source)
            }
            r => Err(ParseError::from_expected(
                "Primary parsing error".to_string(),
                vec![
                    Rule::FunctionCall,
                    Rule::Literal,
                    Rule::Identifier,
                    Rule::Expr,
                ],
                vec![r],
                &AstSpan::from_span(primary.as_span(), source),
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
                    &AstSpan::from_span(op.as_span(), source),
                ))?,
            };
            Ok(AstNode::from_pair(
                Expr::unknown(ExprKind::Binary {
                    op: bin_op,
                    left: Box::new(lhs?),
                    right: Box::new(rhs?),
                }),
                op,
                source,
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
                    &AstSpan::from_span(op.as_span(), source),
                ))?,
            };

            Ok(AstNode::from_pair(
                Expr::unknown(ExprKind::Unary {
                    op: un_op,
                    expr: Box::new(rhs?),
                }),
                op,
                source,
            ))
        })
        .parse(pairs.filter(|x| !matches!(x.as_rule(), Rule::COMMENT)))
}

fn parse_literal(pair: Pair<Rule>, source: &Source) -> Result<AstNode<Expr>> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::Int => Ok(AstNode::from_pair(
            Expr::new(
                ExprKind::Literal,
                Type::Int,
                ExprValue::Int(inner.as_str().parse().unwrap()),
            ),
            inner,
            source,
        )),
        Rule::Hexadecimal => parse_hexadecimal(inner, source),
        Rule::Bool => Ok(AstNode::from_pair(
            Expr::new(
                ExprKind::Literal,
                Type::Bool,
                ExprValue::Bool(inner.as_str() == "true"),
            ),
            inner,
            source,
        )),
        Rule::String => {
            let s = inner.as_str();
            // removes "" quotes
            Ok(AstNode::from_pair(
                Expr::new(
                    ExprKind::Literal,
                    Type::String,
                    ExprValue::String(s[1..s.len() - 1].to_string()),
                ),
                inner,
                source,
            ))
        }
        Rule::Character => {
            let s = inner.as_str();
            let c = unescape_char(&s[1..s.len() - 1]);

            Ok(AstNode::from_pair(
                Expr::new(ExprKind::Literal, Type::Byte, ExprValue::Byte(c as u8)),
                inner,
                source,
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
            &AstSpan::from_span(inner.as_span(), source),
        ))?,
    }
}

fn parse_hexadecimal(pair: Pair<Rule>, source: &Source) -> Result<AstNode<Expr>> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::HexadecimalDigit => Ok(AstNode::from_pair(
            Expr::new(
                ExprKind::Literal,
                Type::Int,
                ExprValue::Int(i32::from_str_radix(inner.as_str(), 16).unwrap()),
            ),
            inner,
            source,
        )),
        r => Err(ParseError::from_expected(
            "Hexadecimal parsing error".to_string(),
            vec![Rule::Int],
            vec![r],
            &AstSpan::from_span(inner.as_span(), source),
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
