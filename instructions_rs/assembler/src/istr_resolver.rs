use std::{collections::HashMap, rc::Rc};

use opcode_gen::instructions::{
    AddressRegister, ArgumentType, Instruction, InstructionSignature, Register,
};

use crate::{
    ast::{AstNode, Statement, TypedExpr},
    types::Type,
};

fn expr_to_argument_type(expr: &AstNode<TypedExpr>) -> ArgumentType {
    let expr = expr.inner();
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

        // coerce labels into address
        Type::Label => ArgumentType::Addr,
        Type::Addr => ArgumentType::Addr,

        // TODO: return error
        _ => todo!(),
    }
    // ensure the type is generic since lookup table is as well
    .to_generic()
}

pub fn resolve_instructions(
    statements: &mut [AstNode<Statement>],
    istr_lookup: &HashMap<InstructionSignature, Rc<Instruction>>,
) {
    for statement in statements {
        match statement.inner_mut() {
            Statement::BlockLabel { body, .. } => {
                resolve_instructions(body, istr_lookup);
            }
            Statement::Instruction(instruction) => {
                let param_types: Vec<ArgumentType> = instruction
                    .params
                    .iter()
                    .map(expr_to_argument_type)
                    .collect();

                let generic_signature =
                    InstructionSignature::new(instruction.name.clone(), param_types);

                if let Some(found_istr) = istr_lookup.get(&generic_signature) {
                    instruction.instruction = Some(Rc::clone(found_istr));
                } else {
                    // TODO: make into error
                    panic!(
                        "Instruction not found from signature: {:#?}",
                        instruction.istr_signature
                    )
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
}

pub fn gen_instruction_lookup_table(
    istrs: &Vec<Rc<Instruction>>,
) -> HashMap<InstructionSignature, Rc<Instruction>> {
    let mut res = HashMap::new();

    for istr in istrs {
        res.insert(
            istr.istr_type().get_signature().to_generic(),
            Rc::clone(istr),
        );
    }

    res
}
