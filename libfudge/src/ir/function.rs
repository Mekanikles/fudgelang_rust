use super::*;

use crate::typesystem::TypeId;
use crate::utils::objectstore::ObjectStore;
use crate::utils::StringKey;

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub basicblockstore: BasicBlockStore,
    pub variablestore: VariableStore,
    pub entry: BasicBlockKey,
}

pub struct FunctionBuilder {
    name: String,
    basicblockstore: BasicBlockStore,
    variablestore: VariableStore,
}

impl FunctionBuilder {
    pub fn new(name: String) -> Self {
        Self {
            name,
            basicblockstore: BasicBlockStore::new(),
            variablestore: VariableStore::new(),
        }
    }

    pub fn create_block(&mut self) -> BasicBlockKey {
        self.basicblockstore.add(BasicBlock::new())
    }

    pub fn edit_block(&mut self, blockkey: &BasicBlockKey) -> BasicBlockEditor {
        BasicBlockEditor {
            block: self.basicblockstore.get_mut(blockkey),
        }
    }

    pub fn get_variable(&self, variable: &VariableKey) -> &Variable {
        self.variablestore.get(&variable)
    }

    pub fn find_last_variable_for_symbol(
        &self,
        block: &BasicBlockKey,
        symbol: &StringKey,
    ) -> Option<VariableKey> {
        // TODO: Search through linked blocks
        let block = self.basicblockstore.get(block);
        for instr in &block.instructions {
            match instr {
                Instruction::Assign(n) => match self.variablestore.get(&n.variable) {
                    Variable::Named {
                        symbol: vsym,
                        typeid: _,
                    } => {
                        if *vsym == *symbol {
                            return Some(n.variable.clone());
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        None
    }

    pub fn add_named_variable(&mut self, symbol: StringKey, typeid: TypeId) -> VariableKey {
        self.variablestore.add(Variable::Named { symbol, typeid })
    }

    pub fn add_unnamed_variable(&mut self, typeid: TypeId) -> VariableKey {
        self.variablestore.add(Variable::Unnamed { typeid })
    }

    pub fn finish(self, entry: BasicBlockKey) -> Function {
        Function {
            name: self.name,
            basicblockstore: self.basicblockstore,
            variablestore: self.variablestore,
            entry,
        }
    }
}
