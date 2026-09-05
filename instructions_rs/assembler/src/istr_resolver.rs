use std::{collections::HashMap, rc::Rc};

use miette::{Result, miette};
use opcode_gen::instructions::{
    AddressRegister, ArgumentType, Instruction, InstructionSignature, Register,
};

use crate::{
    ast::{AstNode, Expr, StatementKind, StatementNode},
    error::ParseError,
    types::Type,
};

fn expr_to_argument_type(expr: &AstNode<Expr>) -> Result<ArgumentType> {
    let inner = expr.inner();
    Ok(match inner.ty {
        // coerce into generic imm since we dont know which it could be
        Type::Int => ArgumentType::GenericImm,

        Type::Register => {
            let name = inner.kind.as_identity().ok_or(ParseError::from_span(
                "Expression with type Register is not identity",
                expr.span(),
            ))?;
            let register = Register::iterator()
                .find(|reg| reg.name() == name.as_str())
                .ok_or(ParseError::from_span(
                    format!("Register not found from name \"{}\"", name),
                    expr.span(),
                ))?;
            ArgumentType::Reg(*register)
        }

        Type::AddressRegister => {
            let name = inner.kind.as_identity().ok_or(ParseError::from_span(
                "Expression with type AddressRegister is not identity",
                expr.span(),
            ))?;
            let register = AddressRegister::iterator()
                .find(|reg| reg.name() == name.as_str())
                .ok_or(ParseError::from_span(
                    format!("AddressRegister not found from name \"{}\"", name),
                    expr.span(),
                ))?;

            ArgumentType::AddrReg(*register)
        }

        // coerce labels into address
        Type::Label => ArgumentType::Addr,
        Type::Addr => ArgumentType::Addr,

        ty => Err(ParseError::from_span(
            format!("Unexpected parameter expression type {:?}", ty),
            expr.span(),
        ))?,
    }
    // ensure the type is generic since lookup table is as well
    .to_generic())
}

pub fn resolve_instructions(
    statements: &mut [StatementNode],
    istr_lookup: &HashMap<InstructionSignature, Rc<Instruction>>,
) -> Result<()> {
    for statement in statements {
        match statement.inner_mut().inner_mut() {
            StatementKind::BlockLabel { body, .. } | StatementKind::Block { body } => {
                resolve_instructions(body, istr_lookup)?;
            }
            StatementKind::Instruction(instruction) => {
                let mut param_types = Vec::new();

                for param in instruction.params.iter() {
                    param_types.push(expr_to_argument_type(param)?);
                }

                let generic_signature =
                    InstructionSignature::new(instruction.name.clone(), param_types);

                if let Some(found_istr) = istr_lookup.get(&generic_signature) {
                    instruction.instruction = Some(Rc::clone(found_istr));
                } else {
                    return Err(ParseError::from_span(
                        format!(
                            "Instruction not found from signature: {:#?}",
                            generic_signature
                        ),
                        statement.span(),
                    )
                    .into());
                }

                // non-generic arguments
                let arguments = instruction
                    .instruction
                    .as_ref()
                    .unwrap()
                    .istr_type()
                    .arguments();

                // set argument types based on the actual signature, to ensure at the eval stage
                // that casting to the correct types is possible
                for (arg, expr) in arguments.iter().zip(instruction.params.iter_mut()) {
                    match arg {
                        ArgumentType::Reg(_) => expr.inner_mut().ty = Type::Register,
                        ArgumentType::AddrReg(_) => expr.inner_mut().ty = Type::AddressRegister,
                        ArgumentType::Byte => expr.inner_mut().ty = Type::Byte,
                        ArgumentType::Addr => expr.inner_mut().ty = Type::Addr,
                        // non-generic should never have generic imm
                        ArgumentType::GenericImm => unreachable!(),
                    }
                }
            }
            _ => (),
        }
    }
    Ok(())
}

pub fn gen_instruction_lookup_table(
    istrs: &Vec<Rc<Instruction>>,
) -> Result<HashMap<InstructionSignature, Rc<Instruction>>> {
    let mut res = HashMap::new();

    for istr in istrs {
        if let Some(other) = res.insert(
            istr.istr_type().get_signature().to_generic(),
            Rc::clone(istr),
        ) {
            return Err(miette!(
                "Duplicate signatures found - \"{}\" and \"{}\"",
                istr,
                other
            ));
        }
    }

    Ok(res)
}
