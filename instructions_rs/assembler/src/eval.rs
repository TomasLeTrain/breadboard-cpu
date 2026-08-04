/// Interpreter for expressions
use std::collections::HashMap;

use crate::ast::{Expr, Statement};

struct Value {}

struct Scope {
    variables: HashMap<Expr, Value>,
}

impl Scope {
    fn new() -> Scope {
        Scope {
            variables: HashMap::new(),
        }
    }
    fn get_variable_value(&self, expr: &Expr) -> Result<&Value, String> {
        assert!(matches!(expr, Expr::Identity(_)));
        self.variables
            .get(expr)
            .ok_or(format!("Variable {:#?} not found", expr))
    }
}

/// fills in expressions
struct Interpreter;

impl Interpreter {
    fn eval_statement(&self, statement: &Statement) {
        match statement {
            Statement::Label { name } => todo!(),
            Statement::BlockLabel { name, body } => todo!(),
            Statement::Instruction { name, params } => todo!(),
        }
    }

    // fn eval_statement(&self, statement: &Statement) {
    //     match statement {
    //         Statement::Label { name } => todo!(),
    //         Statement::BlockLabel { name, body } => todo!(),
    //         Statement::Instruction { name, params } => todo!(),
    //     }
    // }
}
