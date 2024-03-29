use super::*;

use crate::utils::objectstore::ObjectStore;

pub struct Program {
    pub functionstore: FunctionStore,
    pub constantdatastore: ConstantDataStore,
    pub init: FunctionKey,
}

pub struct ProgramBuilder {
    functionstore: FunctionStore,
    constantdatastore: ConstantDataStore,
}

impl ProgramBuilder {
    pub fn new() -> Self {
        Self {
            functionstore: FunctionStore::new(),
            constantdatastore: ConstantDataStore::new(),
        }
    }

    pub fn finish(self, init: FunctionKey) -> Program {
        Program {
            functionstore: self.functionstore,
            constantdatastore: self.constantdatastore,
            init,
        }
    }

    pub fn add_function(&mut self, function: Function) -> FunctionKey {
        self.functionstore.add(function)
    }

    pub fn add_constantdata(&mut self, constantdata: ConstantData) -> ConstantDataKey {
        // TODO: We need to track usage of const data, in case of optimizing away instructions
        self.constantdatastore.add(constantdata)
    }
}

pub fn print_program(program: &Program) {
    println!("  Constant data:");
    for datakey in program.constantdatastore.keys() {
        fn data_to_string(data: &Vec<u8>, caplength: usize) -> String {
            let mut ret = String::new();

            for b in &data[0..caplength] {
                ret.push(if (*b as char).is_ascii_graphic() {
                    *b as char
                } else if (*b as char) == ' ' {
                    ' '
                } else {
                    '·'
                });
            }
            ret
        }

        let data = program.constantdatastore.get(&datakey);

        let header = format!("c{} - {}", datakey, data_to_string(&data.data, 69));
        println!(
            "    {: <79}// {} (size:{})",
            header,
            data.typeid.to_string(),
            data.data.len()
        );
    }

    println!("  Functions:");
    for functionkey in program.functionstore.keys() {
        let function = program.functionstore.get(&functionkey);

        println!(
            "    f{} - {}, entry: b{}",
            functionkey, function.name, function.entry
        );
        for blockkey in function.basicblockstore.keys() {
            let block = function.basicblockstore.get(&blockkey);

            println!(
                "      b{} - declarations: {}, uses: {}, incoming blocks: {}",
                blockkey,
                block.variable_declarations.len(),
                block.variable_usage.len(),
                block.incoming_blocks.len()
            );
            for instr in &block.instructions {
                fn resolve_rhs_variablekey(
                    function: &Function,
                    variablekey: VariableKey,
                ) -> VariableKey {
                    let var = function.variablestore.get(&variablekey);
                    match var {
                        Variable::Substituted { variablekey } => {
                            resolve_rhs_variablekey(function, *variablekey)
                        }
                        _ => variablekey,
                    }
                }

                fn value_to_string(function: &Function, value: &Value) -> String {
                    match value {
                        Value::Primitive { ptype, data } => {
                            format!("{}:{}", ptype.data_to_string(*data), ptype.to_str())
                        }
                        Value::BuiltInFunction { builtin } => {
                            format!("#{:?}", builtin)
                        }
                        Value::TypedValue { typeid, variable } => {
                            format!(
                                "dyn(v{}):{}",
                                resolve_rhs_variablekey(function, *variable),
                                typeid.to_string()
                            )
                        }
                    }
                }

                fn expression_to_string(function: &Function, expr: &Expression) -> String {
                    match expr {
                        Expression::Variable(n) => format!("v{}", n),
                        Expression::Constant(n) => format!("{}", value_to_string(function, n)),
                    }
                }

                fn call_args_to_string(function: &Function, args: &Vec<VariableKey>) -> String {
                    let mut ret = String::new();
                    if !args.is_empty() {
                        for arg in &args[0..args.len() - 1] {
                            ret +=
                                format!("v{}, ", resolve_rhs_variablekey(function, *arg)).as_str();
                        }

                        ret += format!("v{}", args.last().unwrap()).as_str();
                    }
                    ret
                }

                fn instruction_to_string(function: &Function, instr: &Instruction) -> String {
                    match instr {
                        Instruction::Assign(n) => {
                            format!(
                                "v{} = {}",
                                n.variable,
                                expression_to_string(function, &n.expression)
                            )
                        }
                        Instruction::CallBuiltIn(n) => {
                            format!(
                                "v{} = {}({})",
                                resolve_rhs_variablekey(function, n.variable),
                                n.builtin.to_str(),
                                call_args_to_string(function, &n.args)
                            )
                        }
                        Instruction::CallStatic(n) => {
                            format!(
                                "v{} = f{}({})",
                                resolve_rhs_variablekey(function, n.variable),
                                n.function,
                                call_args_to_string(function, &n.args)
                            )
                        }
                        Instruction::Return(n) => {
                            format!("return {}", call_args_to_string(function, &n.values))
                        }
                        Instruction::Halt => {
                            format!("halt")
                        }
                        Instruction::Noop => panic!(),
                    }
                }

                fn print_variable_target_intr(
                    instr: &Instruction,
                    var: &VariableKey,
                    function: &Function,
                ) {
                    println!(
                        "        {: <75}// {}",
                        instruction_to_string(function, instr),
                        function
                            .variablestore
                            .get(var)
                            .get_type(&function.variablestore)
                            .to_string(),
                    );
                }

                match &instr {
                    Instruction::Assign(n) => {
                        print_variable_target_intr(instr, &n.variable, function)
                    }
                    Instruction::CallBuiltIn(n) => {
                        print_variable_target_intr(instr, &n.variable, function)
                    }
                    Instruction::CallStatic(n) => {
                        print_variable_target_intr(instr, &n.variable, function)
                    }
                    Instruction::Noop => (), // Just skip noops
                    _ => {
                        println!("        {}", instruction_to_string(function, instr));
                    }
                }
            }
        }
    }
}
