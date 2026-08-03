use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;

use crate::{
    ast::{Expr, Program, Statement, TypedExpr},
    types::Type,
};

#[derive(Parser)]
#[grammar = "grammar.pest"] // relative to src
pub struct AssemblyParser;

/// Parse source code into a program (list of top-level items)
pub fn parse(source: &str) -> Result<Program, String> {
    let pairs =
        AssemblyParser::parse(Rule::Program, source).map_err(|e| format!("Parse error: {}", e))?;
    println!("{:#?}", pairs);

    let mut program = Vec::new();
    for pair in pairs {
        match pair.as_rule() {
            Rule::Statement => {
                println!("found statement");
                program.push(parse_statement(pair)?);
            }
            Rule::EOI => (),
            _ => {}
        }
    }

    Ok(program)
}

fn parse_statement(pair: Pair<Rule>) -> Result<Statement, String> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::InstructionStatement => Ok(parse_instruction(inner)?),
        Rule::LabelStatement => Ok(Statement::Label {
            name: inner.as_str().to_string(),
        }),
        Rule::BlockLabel => Ok(Statement::BlockLabel {
            name: inner.as_str().to_string(),
            body: parse_block(inner)?,
        }),
        r => Err(format!("Unexpected statement rule: {:?}", r)),
    }
}

fn parse_block(pair: Pair<Rule>) -> Result<Vec<Statement>, String> {
    let mut stmts = Vec::new();
    for item in pair.into_inner() {
        if item.as_rule() == Rule::Statement {
            stmts.push(parse_statement(item)?);
        }
    }
    Ok(stmts)
}

fn parse_instruction(pair: Pair<Rule>) -> Result<Statement, String> {
    let mut name = String::new();
    let mut params = Vec::new();

    for item in pair.into_inner() {
        println!("istr item: {:#?}", item);
        match item.as_rule() {
            Rule::InstructionLabel => name = item.to_string(),
            Rule::InstructionParameters => params = parse_instruction_parameters(item)?,
            r => return Err(format!("Unexpected instruction rule: {:?}", r)),
        };
    }

    Ok(Statement::Instruction { name, params })
}

fn parse_instruction_parameters(pair: Pair<Rule>) -> Result<Vec<TypedExpr>, String> {
    let mut params = Vec::new();

    for item in pair.into_inner() {
        println!("istr parameter item: {:#?}", item);
        match item.as_rule() {
            Rule::InstructionParameter => params.push(parse_expr(item)?),
            r => return Err(format!("Unexpected itsr parameter rule: {:?}", r)),
        };
    }

    Ok(params)
}

fn parse_expr(pair: Pair<Rule>) -> Result<TypedExpr, String> {
    Ok(TypedExpr {
        expr: Expr::Int(0),
        ty: Type::Int,
    })
}

