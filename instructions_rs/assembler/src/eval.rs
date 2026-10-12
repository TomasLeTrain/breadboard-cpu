use std::{collections::HashMap, error::Error, fmt::Display, sync::Arc};

use miette::{Context, Diagnostic, IntoDiagnostic, LabeledSpan, NamedSource, Result, miette};
use opcode_gen::instructions::{AddressRegister, ArgumentValue, Register};

use crate::{
    ast::{
        AstNode, AstSpan, BinaryOp, Expr, ExprKind, Function, FunctionCall, InlinedFunctionCall,
        ReturnKind, Statement, StatementKind, StatementNode, TypedParameter, UnaryOp, Variable,
        VariableExprKind,
    },
    eval,
    types::{Address, Type},
};

#[derive(Debug, Clone)]
pub enum ExprValue {
    Int(i32),
    Bool(bool),
    String(String),

    Register(Register),
    AddressRegister(AddressRegister),

    Addr(Address),
    Byte(u8),
    Block(Vec<StatementNode>),

    Unknown,
}

impl ExprValue {
    fn as_int(&self) -> Option<i32> {
        match self {
            ExprValue::Int(val) => Some(*val),
            ExprValue::Addr(val) => Some(*val as i32),
            ExprValue::Byte(val) => Some(*val as i32),
            ExprValue::Block(_)
            | ExprValue::Register(_)
            | ExprValue::AddressRegister(_)
            | ExprValue::Unknown
            | ExprValue::Bool(_)
            | ExprValue::String(_) => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            ExprValue::Bool(val) => Some(*val),
            ExprValue::Block(_)
            | ExprValue::Int(_)
            | ExprValue::Addr(_)
            | ExprValue::Byte(_)
            | ExprValue::Register(_)
            | ExprValue::AddressRegister(_)
            | ExprValue::Unknown
            | ExprValue::String(_) => None,
        }
    }

    fn apply_int_binary_op(lhs: &Self, rhs: &Self, op: &BinaryOp) -> Self {
        let lhs = lhs.as_int().unwrap();
        let rhs = rhs.as_int().unwrap();

        Self::Int(match op {
            BinaryOp::Add => lhs + rhs,
            BinaryOp::Sub => lhs - rhs,
            BinaryOp::Mul => lhs * rhs,
            BinaryOp::Div => lhs / rhs,
            BinaryOp::Mod => lhs % rhs,
            BinaryOp::Pow => lhs.pow(rhs.try_into().unwrap()),
            BinaryOp::ShiftLeft => lhs << rhs,
            BinaryOp::ShiftRight => lhs >> rhs,
            BinaryOp::BitAnd => lhs & rhs,
            BinaryOp::BitXor => lhs ^ rhs,
            BinaryOp::BitOr => lhs | rhs,
            // TODO: turn into error
            _ => unreachable!(),
        })
    }

    fn apply_bool_binary_op(lhs: &Self, rhs: &Self, op: &BinaryOp) -> Self {
        let lhs = lhs.as_bool().unwrap();
        let rhs = rhs.as_bool().unwrap();

        Self::Bool(match op {
            BinaryOp::And => lhs && rhs,
            BinaryOp::Or => lhs || rhs,
            // TODO: turn into error
            _ => unreachable!(),
        })
    }

    fn apply_comparison_op(lhs: &Self, rhs: &Self, op: &BinaryOp) -> Self {
        // TODO: add support for other comparison types
        let lhs = lhs.as_int().unwrap();
        let rhs = rhs.as_int().unwrap();

        Self::Bool(match op {
            BinaryOp::Lt => lhs < rhs,
            BinaryOp::Gt => lhs > rhs,
            BinaryOp::Le => lhs <= rhs,
            BinaryOp::Ge => lhs >= rhs,
            // TODO: turn into error
            _ => unreachable!(),
        })
    }

    fn apply_equality_op(lhs: &Self, rhs: &Self, op: &BinaryOp) -> Self {
        // TODO: add support for other comparison types
        // TODO: make checking of both types as compatible more robust
        let equal = if lhs.as_int().is_some() {
            lhs.as_int().unwrap() == rhs.as_int().unwrap()
        } else if lhs.as_bool().is_some() {
            lhs.as_bool().unwrap() == rhs.as_bool().unwrap()
        } else {
            // TODO: make into error
            unreachable!();
        };

        Self::Bool(match op {
            BinaryOp::Eq => equal,
            BinaryOp::Ne => !equal,
            // TODO: turn into error
            _ => unreachable!(),
        })
    }

    // TODO: add is_castable function to catch errors before calling this func
    fn cast_value(self, ty: &Type) -> Self {
        // unknown casts to everything
        if matches!(self, Self::Unknown) {
            return self;
        }

        match ty {
            Type::Bool => {
                assert!(matches!(self, Self::Bool(_)));
                self
            }

            Type::Int => {
                // casting from any int-able value
                Self::Int(self.as_int().unwrap())
            }
            Type::Addr => {
                // casting from any int-able value
                Self::Addr(self.as_int().unwrap().try_into().unwrap())
            }
            Type::Byte => {
                // casting from any int-able value
                // TODO: disallow loose casting
                Self::Byte(self.as_int().unwrap().try_into().unwrap())
            }

            Type::Label => {
                assert!(matches!(self, Self::Addr(_)));
                self
            }

            Type::Register => {
                assert!(matches!(self, Self::Register(_)));
                self
            }
            Type::AddressRegister => {
                assert!(matches!(self, Self::AddressRegister(_)));
                self
            }

            Type::String => {
                assert!(matches!(self, Self::String(_)));
                self
            }

            Type::Unknown => unreachable!(),
            Type::Function { .. } => unreachable!(),
            Type::Block => unreachable!(),
        }
    }

    pub fn as_istr_arg_value(&self) -> ArgumentValue {
        match self {
            ExprValue::Register(register) => ArgumentValue::Reg(*register),
            ExprValue::AddressRegister(address_register) => {
                ArgumentValue::AddrReg(*address_register)
            }
            ExprValue::Addr(addr) => ArgumentValue::Addr(*addr),
            ExprValue::Byte(byte) => ArgumentValue::Byte(*byte),

            // TODO: add errors
            ExprValue::Unknown => panic!(),
            ExprValue::Int(_) => panic!(),
            ExprValue::Bool(_) => panic!(),
            ExprValue::String(_) => panic!(),
            ExprValue::Block(_) => todo!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EvalSymbol {
    pub name: String,
    pub ty: Type,
    pub value: ExprValue,
    pub span: Option<AstSpan>,
}

#[derive(Debug, Clone)]
pub enum EvalFunctionKind {
    Function(Function),
    Internal,
}

impl EvalFunctionKind {
    pub fn return_ty(&self) -> &Type {
        match self {
            EvalFunctionKind::Function(function) => &function.return_ty,
            EvalFunctionKind::Internal => todo!(),
        }
    }

    pub fn name(&self) -> &String {
        match self {
            EvalFunctionKind::Function(function) => &function.name,
            EvalFunctionKind::Internal => todo!(),
        }
    }

    pub fn params(&self) -> &Vec<AstNode<TypedParameter>> {
        match self {
            EvalFunctionKind::Function(function) => &function.params,
            EvalFunctionKind::Internal => todo!(),
        }
    }

    // runs with given parameters
    // can return Unknown value if params or the function itself has unkowns
    pub fn eval(&self, params: Vec<EvalSymbol>, ctx: &EvalContext) -> Result<ExprValue> {
        match self {
            EvalFunctionKind::Function(function) => {
                let mut curr_func = function.clone();
                let mut ctx = ctx.clone_non_local();
                let param_names: Vec<String> = params.iter().map(|e| e.name.clone()).collect();

                for param in params {
                    ctx.push_local(param)?;
                }

                let eval_value = eval_program(&mut curr_func.body, &mut ctx)?;

                for name in param_names.into_iter().rev() {
                    if name != ctx.pop_local()?.name {
                        // TODO: make detailed error
                        Err(miette!("popped parameter symbol does not match pushed"))?;
                    }
                }

                Ok(eval_value)
            }
            EvalFunctionKind::Internal => todo!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FunctionEvalSymbol {
    pub name: String,
    pub inner: EvalFunctionKind,
    pub span: Option<AstSpan>,
}

impl FunctionEvalSymbol {
    pub fn new(name: String, inner: EvalFunctionKind, span: Option<AstSpan>) -> Self {
        Self { name, inner, span }
    }

    pub fn inner(&self) -> &EvalFunctionKind {
        &self.inner
    }

    pub fn into_inner(&self) -> &EvalFunctionKind {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut EvalFunctionKind {
        &mut self.inner
    }
}

// keeps track of symbols by keeping track of their scope as well
// allows reusing one context struct through all operations
pub struct EvalContext {
    local_ctx: SymbolContext,
    global_ctx: SymbolContext,

    function_stack: Vec<FunctionEvalSymbol>,
    functions: HashMap<String, FunctionEvalSymbol>,

    is_macro: bool,
}

#[derive(Debug, Clone)]
struct SymbolContext {
    symbol_stack: Vec<EvalSymbol>,
    symbols: HashMap<String, EvalSymbol>,
}

impl SymbolContext {
    fn new() -> Self {
        Self {
            symbol_stack: Vec::new(),
            symbols: HashMap::new(),
        }
    }

    fn push(&mut self, symbol: EvalSymbol) -> Result<()> {
        self.symbol_stack.push(symbol.clone());

        let push_result = self.symbols.insert(symbol.clone().name, symbol.clone());

        if let Some(other) = push_result {
            DuplicateSymbolError::from_symbols(symbol, other)
        } else {
            Ok(())
        }
    }

    fn pop(&mut self) -> Result<EvalSymbol> {
        let popped_symbol = self
            .symbol_stack
            .pop()
            .ok_or(EmptyStackError {})
            .into_diagnostic()?;

        let map_symbol = self.symbols.remove(&popped_symbol.name).unwrap();

        Ok(map_symbol)
    }

    fn get(&self, name: &String) -> Option<&EvalSymbol> {
        self.symbols.get(name)
    }

    fn contains(&self, name: &String) -> bool {
        self.symbols.contains_key(name)
    }
}

impl EvalContext {
    pub fn new(is_macro: bool) -> Self {
        EvalContext {
            global_ctx: SymbolContext::new(),
            local_ctx: SymbolContext::new(),
            function_stack: Vec::new(),
            functions: HashMap::new(),
            is_macro,
        }
    }

    pub fn clone_non_local(&self) -> Self {
        Self {
            global_ctx: self.global_ctx.clone(),
            local_ctx: SymbolContext::new(),
            function_stack: self.function_stack.clone(),
            functions: self.functions.clone(),
            is_macro: self.is_macro(),
        }
    }

    pub fn push_global(&mut self, symbol: EvalSymbol) -> Result<()> {
        // must check if its in either local or global context to avoid conflicts bewteen them
        if self.contains(&symbol.name) {
            let other = self.get(&symbol.name).unwrap().clone();
            return DuplicateSymbolError::from_symbols(symbol, other);
        }

        self.global_ctx.push(symbol)
    }

    fn push_local(&mut self, symbol: EvalSymbol) -> Result<()> {
        // must check if its in either local or global context to avoid conflicts bewteen them
        if self.contains(&symbol.name) {
            let other = self.get(&symbol.name).unwrap().clone();
            return DuplicateSymbolError::from_symbols(symbol, other);
        }

        self.local_ctx.push(symbol)
    }

    fn pop_local(&mut self) -> Result<EvalSymbol> {
        self.local_ctx.pop()
    }

    fn get(&self, name: &String) -> Option<&EvalSymbol> {
        self.local_ctx.get(name).or(self.global_ctx.get(name))
    }

    fn contains(&self, name: &String) -> bool {
        self.local_ctx.contains(name) | self.global_ctx.contains(name)
    }

    pub fn push_function(&mut self, function: FunctionEvalSymbol) -> Result<()> {
        self.function_stack.push(function.clone());

        let inner = function.inner();

        let push_result = self
            .functions
            .insert(inner.name().clone(), function.clone());

        if let Some(other) = push_result {
            let mut spans = Vec::new();
            let source = function.span.as_ref().map(|e| e.to_miette_source_code());

            for ast_span in [function.span.as_ref(), other.span.as_ref()]
                .into_iter()
                .flatten()
            {
                spans.push(LabeledSpan::new_with_span(
                    Some(format!(
                        "Function \"{}\" of params \"{:?}\" and return type \"{:?}\" defined here",
                        inner.name(),
                        inner.params(),
                        inner.return_ty()
                    )),
                    ast_span.to_miette_span(),
                ));
            }

            Err(DuplicateSymbolError {
                name: function.name,
                source,
                spans,
            })?
        } else {
            Ok(())
        }
    }

    fn pop_function(&mut self) -> Result<FunctionEvalSymbol> {
        let popped_symbol = self
            .function_stack
            .pop()
            // TODO: add type of stack to error
            .ok_or(EmptyStackError {})
            .into_diagnostic()?;

        let map_function = self.functions.remove(&popped_symbol.name).unwrap();

        Ok(map_function)
    }

    fn get_function(&self, name: &String) -> Option<&FunctionEvalSymbol> {
        self.functions.get(name)
    }

    fn contains_function(&self, name: &String) -> bool {
        self.functions.contains_key(name)
    }

    pub fn is_macro(&self) -> bool {
        self.is_macro
    }
}

pub fn eval_program(statements: &mut [StatementNode], ctx: &mut EvalContext) -> Result<ExprValue> {
    let mut local_symbols = Vec::new();
    let mut function_symbols = Vec::new();

    let mut return_value = ExprValue::Unknown;
    let mut return_hit = false;

    // first find all labels in the current scope (accessible from anywhere in scope)
    for statement in statements.iter() {
        match statement.inner().inner() {
            StatementKind::Label { name } | StatementKind::BlockLabel { name, .. } => {
                // push into local scope
                let curr_symbol = EvalSymbol {
                    name: name.clone(),
                    ty: Type::Label,
                    value: statement
                        .inner()
                        .address()
                        .map(ExprValue::Addr)
                        .unwrap_or(ExprValue::Unknown),
                    span: Some(statement.span().clone()),
                };

                ctx.push_local(curr_symbol.clone())
                    .wrap_err("Pushing local label symbol failed.")?;

                local_symbols.push(curr_symbol);
            }
            StatementKind::Function(function) => {
                let function_symbol = FunctionEvalSymbol::new(
                    function.name.clone(),
                    EvalFunctionKind::Function(function.clone()),
                    Some(statement.span().clone()),
                );
                ctx.push_function(function_symbol.clone())
                    .wrap_err("Pushing function failed.")?;
                function_symbols.push(function_symbol);
            }
            _ => (),
        }
    }

    // then eval everything else
    for statement in statements.iter_mut() {
        // TODO: should already be caught by typecheck
        if return_hit {
            Err(miette!("unreachable code after return statement!"))?;
        }
        match statement.inner_mut().inner_mut() {
            StatementKind::Variable(Variable {
                name,
                expr_kind,
                ty,
            }) => {
                let value = match expr_kind {
                    VariableExprKind::Expr(expr) => {
                        eval_expr(expr, ctx)?;
                        expr.inner.value.clone()
                    }
                    VariableExprKind::Block(block) => ExprValue::Block(block.clone()),
                };

                // push into local scope
                let curr_symbol = EvalSymbol {
                    name: name.clone(),
                    ty: ty.clone(),
                    value,
                    span: Some(statement.span().clone()),
                };

                ctx.push_local(curr_symbol.clone())
                    .wrap_err("Pushing local variable symbol failed.")?;
                local_symbols.push(curr_symbol);
            }
            StatementKind::Return(return_kind) => {
                match return_kind {
                    ReturnKind::Expr(expr) => {
                        eval_expr(expr, ctx)?;
                        return_value = expr.inner().value.clone();
                    }
                    ReturnKind::Block(block) => {
                        let mut new_block = block.clone();
                        eval_program(&mut new_block, ctx)?;
                        return_value = ExprValue::Block(new_block)
                    }
                }
                return_hit = true;
            }

            StatementKind::InlinedFunctionCall(InlinedFunctionCall { name, params, body }) => {
                // eval function first, then reuse that context for the body itself
                // this assumes that the value returned by eval_program(function.body,ctx) = body, except
                // body has addresses allocated

                if !ctx.contains_function(name) {
                    // TODO: detailed error
                    Err(miette!("function from function call not found"))?;
                }

                // eval all params
                for param in params.iter_mut() {
                    eval_expr(param, ctx)?;

                    if !ctx.is_macro() && matches!(param.inner().value, ExprValue::Unknown) {
                        // TODO: detailed error
                        Err(miette!("function parameter unknown on final eval stage"))?;
                    }
                }

                let function_symbol = ctx.get_function(name).unwrap();

                let valued_params: Vec<EvalSymbol> = params
                    .iter()
                    .zip(function_symbol.inner().params())
                    .map(|(expr, typed_param)| EvalSymbol {
                        name: typed_param.inner().name.clone(),
                        ty: expr.inner().ty.clone(),
                        value: expr.inner().value.clone(),
                        span: Some(expr.span().clone()),
                    })
                    .collect();

                // evaluate function, only for collecting context
                if let EvalFunctionKind::Function(function) = function_symbol.inner() {
                    let mut curr_func = function.clone();
                    let mut ctx = ctx.clone_non_local();

                    let param_names: Vec<String> =
                        valued_params.iter().map(|e| e.name.clone()).collect();

                    for param in valued_params {
                        ctx.push_local(param)?;
                    }

                    // only eval to collect context, what this returns should equal the inlined
                    // function call body but without allocated addresses
                    eval_program(&mut curr_func.body, &mut ctx)?;

                    // now eval the inlined function call body with the context after the function ends
                    eval_program(body, &mut ctx)?;

                    for name in param_names.into_iter().rev() {
                        if name != ctx.pop_local()?.name {
                            // TODO: make detailed error
                            Err(miette!("popped parameter symbol does not match pushed"))?;
                        }
                    }
                } else {
                    // TODO: detailed error
                    // should never happen since its an inlined/block function
                    unreachable!()
                }
            }

            StatementKind::BlockLabel { body, .. } | StatementKind::Block { body } => {
                eval_program(body, ctx)?;
            }
            // instructions not evaled on macro
            StatementKind::Instruction(instruction) => {
                for param in instruction.params.iter_mut() {
                    eval_expr(param, ctx)?;
                }
            }
            _ => (),
        };

        if matches!(statement.inner().inner(), StatementKind::FunctionCall(_)) {
            // clone the statement to be able to replace original
            let mut new_statement = statement.clone();

            if let StatementKind::FunctionCall(FunctionCall { name, params }) =
                new_statement.inner_mut().inner_mut()
            {
                if !ctx.is_macro() {
                    // TODO: detailed error
                    Err(miette!("function call found on normal eval"))?;
                }

                if !ctx.contains_function(name) {
                    // TODO: detailed error
                    Err(miette!("function from function call not found"))?;
                }

                // eval all params
                for param in params.iter_mut() {
                    eval_expr(param, ctx)?;

                    if !ctx.is_macro() && matches!(param.inner().value, ExprValue::Unknown) {
                        // TODO: detailed error
                        Err(miette!("function parameter unknown on final eval stage"))?;
                    }
                }

                let function_symbol = ctx.get_function(name).unwrap();

                let valued_params: Vec<EvalSymbol> = params
                    .iter()
                    .zip(function_symbol.inner().params())
                    .map(|(expr, typed_param)| EvalSymbol {
                        name: typed_param.inner().name.clone(),
                        ty: expr.inner().ty.clone(),
                        value: expr.inner().value.clone(),
                        span: Some(expr.span().clone()),
                    })
                    .collect();

                // evaluate function
                let eval_value = function_symbol.inner().eval(valued_params, ctx)?;

                if !ctx.is_macro() && matches!(eval_value, ExprValue::Unknown) {
                    // TODO: detailed error
                    Err(miette!("function returned unknown on final eval stage"))?;
                }

                let block = if let ExprValue::Block(eval_block) = eval_value {
                    eval_block
                } else {
                    // TODO: detailed error
                    Err(miette!(
                        "expected block from function call eval, got else instead"
                    ))?;
                    unreachable!();
                };

                // now, replace the current statement with the inlined function call
                *statement = StatementNode::new(
                    Statement::new(StatementKind::InlinedFunctionCall(
                        InlinedFunctionCall::new(name.clone(), params.clone(), block),
                    )),
                    new_statement.span().clone(),
                );
            }
        }
    }

    // checks that all returned symbols match what was pushed in
    // goes in reverse since pop starts from the last added element
    for label in local_symbols.into_iter().rev() {
        let curr = ctx.pop_local()?;
        if label.name != curr.name {
            return Err(miette!(
                "Popped symbol does not match - original: {:?}, got: {:?}",
                label,
                curr,
            ));
        }
    }

    for label in function_symbols.into_iter().rev() {
        let curr = ctx.pop_function()?;
        if label.name != curr.name {
            return Err(miette!(
                "Popped symbol does not match - original: {:?}, got: {:?}",
                label,
                curr,
            ));
        }
    }

    Ok(return_value)
}

fn eval_expr(typed_expr: &mut AstNode<Expr>, ctx: &mut EvalContext) -> Result<()> {
    let inner_span = typed_expr.span().clone();
    let inner = typed_expr.inner_mut();

    // expr already evaluated, must still cast in case its a literal
    if !matches!(inner.value, ExprValue::Unknown) {
        inner.value = inner.value.clone().cast_value(&inner.ty);
        return Ok(());
    }

    match &mut inner.kind {
        // literals already have their value filled in
        ExprKind::Literal => (),
        ExprKind::FunctionCall(FunctionCall { name, params }) => {
            if !ctx.contains_function(name) {
                // TODO: detailed error
                Err(miette!("function from function call not found"))?;
            }

            // eval all params
            for param in params.iter_mut() {
                eval_expr(param, ctx)?;

                if !ctx.is_macro() && matches!(param.inner().value, ExprValue::Unknown) {
                    // TODO: detailed error
                    Err(miette!("function parameter unknown on final eval stage"))?;
                }
            }

            let function_symbol = ctx.get_function(name).unwrap();

            let valued_params: Vec<EvalSymbol> = params
                .iter()
                .zip(function_symbol.inner().params())
                .map(|(expr, typed_param)| EvalSymbol {
                    name: typed_param.inner().name.clone(),
                    ty: expr.inner().ty.clone(),
                    value: expr.inner().value.clone(),
                    span: Some(expr.span().clone()),
                })
                .collect();

            // evaluate function
            let eval_value = function_symbol.inner().eval(valued_params, ctx)?;

            if !ctx.is_macro() && matches!(eval_value, ExprValue::Unknown) {
                // TODO: detailed error
                Err(miette!("function returned unknown on final eval stage"))?;
            }

            inner.value = eval_value.cast_value(&inner.ty);
        }
        ExprKind::Identity(name) => {
            if ctx.contains(name) {
                let symbol = ctx.get(name).unwrap();

                inner.value = symbol.value.clone().cast_value(&inner.ty);
            } else {
                Err(EvalExprError::new(EvalExprErrorKind::SymbolNotFound(
                    EvalSymbol {
                        name: name.to_string(),
                        ty: inner.ty.clone(),
                        span: Some(inner_span),
                        value: ExprValue::Unknown,
                    },
                )))?;
            }
        }
        ExprKind::Unary {
            op,
            expr: unary_expr,
        } => {
            eval_expr(unary_expr, ctx)?;
            let unary_expr = unary_expr.inner_mut();

            inner.value = match op {
                UnaryOp::Neg => ExprValue::Int(-unary_expr.value.as_int().unwrap()),
                UnaryOp::BitNegation => ExprValue::Int(!unary_expr.value.as_int().unwrap()),
                UnaryOp::Not => ExprValue::Bool(!unary_expr.value.as_bool().unwrap()),
            };
        }
        ExprKind::Binary { op, left, right } => {
            eval_expr(left, ctx)?;
            eval_expr(right, ctx)?;

            let left = left.inner_mut();
            let right = right.inner_mut();

            inner.value = match op {
                BinaryOp::Add
                | BinaryOp::Sub
                | BinaryOp::Mul
                | BinaryOp::Div
                | BinaryOp::Mod
                | BinaryOp::Pow
                | BinaryOp::ShiftLeft
                | BinaryOp::ShiftRight
                | BinaryOp::BitAnd
                | BinaryOp::BitXor
                | BinaryOp::BitOr => ExprValue::apply_int_binary_op(&left.value, &right.value, op),
                BinaryOp::And | BinaryOp::Or => {
                    ExprValue::apply_bool_binary_op(&left.value, &right.value, op)
                }
                BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge => {
                    ExprValue::apply_comparison_op(&left.value, &right.value, op)
                }
                BinaryOp::Eq | BinaryOp::Ne => {
                    ExprValue::apply_equality_op(&left.value, &right.value, op)
                }
            }
        }
    }

    // ensure value confines to the type
    inner.value = inner.value.clone().cast_value(&inner.ty);

    Ok(())
}

#[derive(Diagnostic, Debug)]
#[diagnostic(code(eval::duplicate_symbol))]
pub struct DuplicateSymbolError {
    name: String,

    #[source_code]
    source: Option<NamedSource<Arc<str>>>,

    #[label(collection, "Defined here")]
    spans: Vec<LabeledSpan>,
}

impl DuplicateSymbolError {
    fn from_symbols(symbol: EvalSymbol, other: EvalSymbol) -> Result<()> {
        let mut spans = Vec::new();

        let source = symbol.span.as_ref().map(|e| e.to_miette_source_code());

        for ast_span in [symbol.span.as_ref(), other.span.as_ref()]
            .into_iter()
            .flatten()
        {
            spans.push(LabeledSpan::new_with_span(
                Some(format!(
                    "Symbol of type \"{:?}\" with value \"{:?}\" defined here",
                    symbol.ty, symbol.value
                )),
                ast_span.to_miette_span(),
            ));
        }

        Err(DuplicateSymbolError {
            name: symbol.name,
            source,
            spans,
        })?
    }
}

impl Error for DuplicateSymbolError {}

impl Display for DuplicateSymbolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Symbol \"{}\" already in context", self.name)
    }
}

#[derive(Debug)]
pub struct EmptyStackError;

impl Error for EmptyStackError {}

impl Display for EmptyStackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "No symbols/functions in stack.")
    }
}

#[derive(Debug)]
pub enum EvalExprErrorKind {
    SymbolNotFound(EvalSymbol),
}

impl EvalExprErrorKind {
    fn get_spans(&self) -> Vec<LabeledSpan> {
        match self {
            EvalExprErrorKind::SymbolNotFound(symbol) => {
                if let Some(span) = &symbol.span {
                    vec![LabeledSpan::new_with_span(
                        Some("Symbol defined here".to_string()),
                        span,
                    )]
                } else {
                    vec![]
                }
            }
        }
    }

    fn get_source(&self) -> Option<NamedSource<Arc<str>>> {
        match self {
            EvalExprErrorKind::SymbolNotFound(symbol) => {
                symbol.span.as_ref().map(AstSpan::to_miette_source_code)
            }
        }
    }
}

impl Display for EvalExprErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalExprErrorKind::SymbolNotFound(symbol) => {
                write!(f, "Symbol \"{}\" not found", symbol.name)
            }
        }
    }
}

#[derive(Diagnostic, Debug)]
pub struct EvalExprError {
    #[source_code]
    source: Option<NamedSource<Arc<str>>>,
    kind: EvalExprErrorKind,

    #[label(collection, "Defined here")]
    spans: Vec<LabeledSpan>,
}

impl Error for EvalExprError {}

impl Display for EvalExprError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl EvalExprError {
    fn new(kind: EvalExprErrorKind) -> Self {
        let spans = kind.get_spans();
        let source = kind.get_source();

        EvalExprError {
            spans,
            source,
            kind,
        }
    }
}
