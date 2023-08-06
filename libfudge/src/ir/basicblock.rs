use crate::asg::expressions::BuiltInFunction;

use super::*;

#[derive(Debug)]
pub struct BasicBlock {
    pub instructions: Vec<Instruction>,
}

impl BasicBlock {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
        }
    }

    pub fn push_instruction(&mut self, instruction: Instruction) {
        self.instructions.push(instruction)
    }
}

pub struct BasicBlockEditor<'a> {
    pub block: &'a mut BasicBlock,
}

impl<'a> BasicBlockEditor<'a> {
    pub fn assign(&mut self, variable: VariableKey, expression: Expression) {
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
        args: Vec<Expression>,
    ) {
        self.block
            .push_instruction(Instruction::CallStatic(instructions::CallStatic {
                variable,
                function,
                args,
            }))
    }

    pub fn call_builtin(
        &mut self,
        variable: VariableKey,
        builtin: crate::typesystem::BuiltInFunction,
        args: Vec<Expression>,
    ) {
        self.block
            .push_instruction(Instruction::CallBuiltIn(instructions::CallBuiltIn {
                variable,
                builtin,
                args,
            }))
    }
}
