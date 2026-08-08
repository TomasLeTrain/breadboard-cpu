use opcode_gen::instructions::{AddressRegister, ArgumentType, InstructionSignature, Register};

use crate::{
    ast::{Statement, TypedExpr},
    types::Type,
};

fn expr_to_argument_type(expr: &TypedExpr) -> ArgumentType {
    match expr.ty {
        // coerce into generic imm since we dont know which it could be
        Type::Int => ArgumentType::GenericImm,
        Type::Register => {
            // TODO: add errors
            let name = expr.expr.as_identity().unwrap();
            let register = Register::iterator()
                .find(|reg| reg.name() == name.as_str())
                .unwrap();
            ArgumentType::Reg(*register)
        }
        Type::AddressRegister => {
            // TODO: add errors
            let name = expr.expr.as_identity().unwrap();
            let register = AddressRegister::iterator()
                .find(|reg| reg.name() == name.as_str())
                .unwrap();
            ArgumentType::AddrReg(*register)
        }
        // coerce into address
        Type::Label => ArgumentType::Addr,
        Type::Unknown => todo!(),
        // TODO: return error
        _ => todo!(),
    }
}

pub fn resolve_istr_signatures(statements: &mut [Statement]) {
    for statement in statements {
        match statement {
            Statement::BlockLabel { body, .. } => {
                resolve_istr_signatures(body);
            }
            Statement::Instruction(instruction) => {
                let param_types: Vec<ArgumentType> = instruction
                    .params
                    .iter()
                    .map(expr_to_argument_type)
                    .collect();

                instruction.istr_signature = Some(InstructionSignature::new(
                    instruction.name.clone(),
                    param_types,
                ));
            }
            _ => (),
        }
    }
}
