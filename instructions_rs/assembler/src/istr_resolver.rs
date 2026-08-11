use std::{collections::HashMap, rc::Rc};

use miette::{Result, miette};
use opcode_gen::instructions::{
    AddressRegister, ArgumentType, Instruction, InstructionSignature, Register,
};

use crate::{
    ast::{AstNode, Statement, TypedExpr},
    error::ParseError,
    types::Type,
};

fn expr_to_argument_type(expr: &AstNode<TypedExpr>) -> Result<ArgumentType> {
    let inner = expr.inner();
    Ok(match inner.ty {
        // coerce into generic imm since we dont know which it could be
        Type::Int => ArgumentType::GenericImm,

        Type::Register => {
            let name = inner.expr.as_identity().ok_or(ParseError::from_span(
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
            let name = inner.expr.as_identity().ok_or(ParseError::from_span(
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
    statements: &mut [AstNode<Statement>],
    istr_lookup: &HashMap<InstructionSignature, Rc<Instruction>>,
) -> Result<()> {
    for statement in statements {
        match statement.inner_mut() {
            Statement::BlockLabel { body, .. } => {
                resolve_instructions(body, istr_lookup)?;
            }
            Statement::Instruction(instruction) => {
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

                // save non-generic instruction
                let signature = instruction
                    .instruction
                    .as_ref()
                    .unwrap()
                    .istr_type()
                    .get_signature();

                instruction.istr_signature = Some(signature);
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
                "Duplicate signatures found - {} and {}",
                istr,
                other
            ));
        }
    }

    Ok(res)
}
