use std::collections::HashMap;

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
    pub variablestore: VariableStore, // TODO: pub
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

    pub fn update_variable_usage(&mut self) {
        fn is_declared(
            zelf: &mut FunctionBuilder,
            blockkey: BasicBlockKey,
            variable: VariableKey,
        ) -> bool {
            zelf.basicblockstore
                .get_mut(&blockkey)
                .variable_declarations
                .contains_key(&variable)
        }

        fn update_block_usage(
            zelf: &mut FunctionBuilder,
            blockkey: BasicBlockKey,
            outgoingkey: BasicBlockKey,
            variable: VariableKey,
        ) {
            let block = zelf.basicblockstore.get_mut(&blockkey);
            let outgoing_usage = if let Some(varusage) = block.variable_usage.get_mut(&variable) {
                &mut varusage.outgoing_usage
            } else {
                block
                    .variable_usage
                    .insert(variable, VariableUsageInfo::default());
                &mut block
                    .variable_usage
                    .get_mut(&variable)
                    .unwrap()
                    .outgoing_usage
            };

            // If we already visited this path, we can end traversal
            if outgoing_usage.contains(&outgoingkey) {
                return;
            } else {
                outgoing_usage.insert(outgoingkey);
            }

            // Continue traversal if the variable was not declared in this block
            if !is_declared(zelf, blockkey, variable) {
                for incoming_blockkey in zelf.basicblockstore.get(&blockkey).incoming_blocks.clone()
                {
                    update_block_usage(zelf, incoming_blockkey, blockkey, variable);
                }
            }
        }

        // Update usage for all variables in all blocks
        for blockkey in self.basicblockstore.keys() {
            let mut checkpair = Vec::new();
            for used_var in &self.basicblockstore.get(&blockkey).variable_usage {
                for incoming_blockkey in &self.basicblockstore.get(&blockkey).incoming_blocks {
                    checkpair.push((*incoming_blockkey, *used_var.0));
                }
            }

            for p in checkpair {
                update_block_usage(self, p.0, blockkey, p.1);
            }
        }
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
