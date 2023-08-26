use std::collections::{HashMap, HashSet};

use crate::asg::expressions::BuiltInFunction;

use super::*;

#[derive(Default, Debug)]
pub struct VariableUsageInfo {
    pub last_usage_point: Option<usize>, // instruction index
    pub outgoing_usage: HashSet<BasicBlockKey>,
}

#[derive(Default, Debug)]
pub struct VariableDeclarationInfo {
    declare_point: usize, // instruction index
}

#[derive(Default, Debug)]
pub struct BasicBlock {
    pub instructions: Vec<Instruction>,
    pub variable_usage: HashMap<VariableKey, VariableUsageInfo>,
    pub variable_declarations: HashMap<VariableKey, VariableDeclarationInfo>,

    pub incoming_blocks: HashSet<BasicBlockKey>,
}

impl BasicBlock {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_instruction(&mut self, instruction: Instruction) {
        self.instructions.push(instruction)
    }

    pub fn add_incoming_block(&mut self, block: BasicBlockKey) {
        self.incoming_blocks.insert(block);
    }

    pub fn add_declaration_on_next_instruction(&mut self, variable: VariableKey) {
        debug_assert!(!self.variable_declarations.contains_key(&variable));

        let next_instruction_index = self.instructions.len();
        self.variable_declarations.insert(
            variable,
            VariableDeclarationInfo {
                declare_point: next_instruction_index,
            },
        );
    }

    pub fn add_usage_on_next_instruction(&mut self, variable: VariableKey) {
        let next_instruction_index = self.instructions.len();
        if let Some(existing) = self.variable_usage.get_mut(&variable) {
            existing.last_usage_point = Some(next_instruction_index);
        } else {
            self.variable_usage.insert(
                variable,
                VariableUsageInfo {
                    last_usage_point: Some(next_instruction_index),
                    outgoing_usage: HashSet::new(),
                },
            );
        }
    }
}

pub struct BasicBlockEditor<'a> {
    pub block: &'a mut BasicBlock,
}

impl<'a> BasicBlockEditor<'a> {
    pub fn assign(&mut self, variable: VariableKey, expression: Expression) {
        self.block.add_declaration_on_next_instruction(variable);

        self.block
            .push_instruction(Instruction::Assign(instructions::Assign {
                variable,
                expression,
            }));
    }

    pub fn call_static(
        &mut self,
        variable: VariableKey,
        function: FunctionKey,
        args: Vec<VariableKey>,
    ) {
        self.block.add_declaration_on_next_instruction(variable);
        for arg in &args {
            self.block.add_usage_on_next_instruction(*arg);
        }

        self.block
            .push_instruction(Instruction::CallStatic(instructions::CallStatic {
                variable,
                function,
                args,
            }));
    }

    pub fn call_builtin(
        &mut self,
        variable: VariableKey,
        builtin: crate::typesystem::BuiltInFunction,
        args: Vec<VariableKey>,
    ) {
        self.block.add_declaration_on_next_instruction(variable);
        for arg in &args {
            self.block.add_usage_on_next_instruction(*arg);
        }

        self.block
            .push_instruction(Instruction::CallBuiltIn(instructions::CallBuiltIn {
                variable,
                builtin,
                args,
            }))
    }

    pub fn do_return(&mut self, values: Vec<VariableKey>) {
        for value in &values {
            self.block.add_usage_on_next_instruction(*value);
        }

        self.block
            .push_instruction(Instruction::Return(instructions::Return { values }));
    }

    pub fn halt(&mut self) {
        self.block.push_instruction(Instruction::Halt);
    }
}
