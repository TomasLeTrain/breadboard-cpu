use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use crate::ast::*;
use crate::types::Type;
use pest::Parser;
use pest::iterators::{Pair, Pairs};
use pest::pratt_parser::{Assoc, Op, PrattParser};

use pest::error::Error;

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

/// Parse source code into a program (list of top-level items)
pub fn parse_file<'a>(source: &'a str, file_path: &'a Path) -> Result<File<'a>, Error> {
    let mut res = File::new(source, file_path);


    let pairs =
        AssemblyParser::parse(Rule::Program, source).map_err(|e| format!("Parse error: {}", e))?;

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

fn parse_statement(pair: Pair<Rule>) -> Result<AstNode<Statement>, String> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::InstructionStatement => parse_instruction(inner),
        Rule::LabelStatement => parse_label(inner),
        Rule::BlockLabel => parse_block_label(inner),
        r => Err(format!("Unexpected statement rule: {:?}", r)),
    }
}

fn parse_label(pair: Pair<Rule>) -> Result<AstNode<Statement>, String> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::LabelIdentifier => Ok(AstNode::from_pair(
            Statement::Label {
                name: inner.to_string(),
            },
            inner,
        )),
        r => Err(format!("Unexpected label rule: {:?}", r)),
    }
}

fn parse_block_label(pair: Pair<Rule>) -> Result<AstNode<Statement>, String> {
    let mut name: Result<String, String> = Err("Name not found".to_string());
    let mut body: Result<Vec<AstNode<Statement>>, String> = Err("Body not found".to_string());
    let span = pair.as_span();

    for item in pair.into_inner() {
        match item.as_rule() {
            Rule::LabelIdentifier => name = Ok(item.to_string()),
            Rule::Block => body = Ok(parse_block(item)?),
            r => return Err(format!("Unexpected label rule: {:?}", r)),
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

fn parse_block(pair: Pair<Rule>) -> Result<Vec<AstNode<Statement>>, String> {
    let mut stmts = Vec::new();
    for item in pair.into_inner() {
        match item.as_rule() {
            Rule::Statement => stmts.push(parse_statement(item)?),
            r => return Err(format!("Unexpected block rule: {:?}", r)),
        }
    }
    Ok(stmts)
}

fn parse_instruction(pair: Pair<Rule>) -> Result<AstNode<Statement>, String> {
    let mut name = String::new();
    let mut params = Vec::new();
    let span = pair.as_span();

    for item in pair.into_inner() {
        // println!("istr item: {:#?}", item);
        match item.as_rule() {
            Rule::InstructionLabel => name = item.to_string(),
            Rule::InstructionParameters => params = parse_instruction_parameters(item)?,
            r => return Err(format!("Unexpected instruction rule: {:?}", r)),
        };
    }

    Ok(AstNode::new(
        Statement::Instruction(AstInstruction::new(name, params)),
        span,
    ))
}

fn parse_instruction_parameters(pair: Pair<Rule>) -> Result<Vec<AstNode<TypedExpr>>, String> {
    let mut params = Vec::new();

    for item in pair.into_inner() {
        // println!("istr parameter item: {:#?}", item);
        match item.as_rule() {
            Rule::Expr => params.push(parse_expr(item.into_inner())?),
            r => return Err(format!("Unexpected itsr parameter rule: {:?}", r)),
        };
    }

    Ok(params)
}

fn parse_expr(pairs: Pairs<Rule>) -> Result<AstNode<TypedExpr>, String> {
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::Literal => parse_literal(primary),
            Rule::Identifier => Ok(AstNode::from_pair(
                TypedExpr::unknown(Expr::Identity(primary.as_str().to_string())),
                primary,
            )),
            Rule::Expr => parse_expr(primary.into_inner()),
            r => Err(format!("Unexpected primary: {:?}", r)),
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
                _ => return Err(format!("Unexpected infix op: {:?}", op)),
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
                _ => return Err(format!("Unexpected prefix op: {:?}", op)),
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

fn parse_literal(pair: Pair<Rule>) -> Result<AstNode<TypedExpr>, String> {
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
        r => Err(format!("Unexpected literal rule: {:?}", r)),
    }
}

fn parse_hexadecimal(pair: Pair<Rule>) -> Result<AstNode<TypedExpr>, String> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::Int => Ok(AstNode::from_pair(
            TypedExpr::new(
                Expr::Int(i64::from_str_radix(inner.as_str(), 16).unwrap()),
                Type::Int,
            ),
            inner,
        )),
        r => Err(format!("Unexpected hexadecimal rule: {:?}", r)),
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
