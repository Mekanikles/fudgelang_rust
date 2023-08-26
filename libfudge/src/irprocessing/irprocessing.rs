use crate::{asg::objectstore::ObjectStore, ir};

pub fn is_trivial_assign(assign: &ir::instructions::Assign) -> Option<ir::VariableKey> {
    match assign.expression {
        ir::Expression::Variable(n) => Some(n),
        _ => None,
    }
}

pub fn substitute_usage(
    block: &mut ir::BasicBlock,
    old_variable: ir::VariableKey,
    new_variable: ir::VariableKey,
) {
    if !block.variable_usage.contains_key(&old_variable) {
        return;
    };

    let old_usage = block.variable_usage.remove(&old_variable).unwrap();

    if let Some(new_usage) = block.variable_usage.get_mut(&new_variable) {
        new_usage.last_usage_point =
            std::cmp::max(old_usage.last_usage_point, new_usage.last_usage_point);
        new_usage.outgoing_usage.extend(&old_usage.outgoing_usage);
    } else {
        // Just swap variables for the existing usage
        block.variable_usage.insert(new_variable, old_usage);
    }
}

pub fn substitute_variable(
    irfunction: &mut ir::Function,
    old_variable: ir::VariableKey,
    new_variable: ir::VariableKey,
) {
    if irfunction.variablestore.has(&old_variable) {
        *irfunction.variablestore.get_mut(&old_variable) = ir::Variable::Substituted {
            variablekey: new_variable,
        }
    }

    for block in irfunction.basicblockstore.values_mut() {
        substitute_usage(block, old_variable, new_variable);
    }
}

pub fn process_function(irfunction: &mut ir::Function) {
    // Find variable substitutions, vx = vy
    let mut variable_substitutions = Vec::new();
    for block in irfunction.basicblockstore.values_mut() {
        for instr in &mut block.instructions {
            match instr {
                ir::Instruction::Assign(n) => {
                    // Remove trivial assigns
                    if let Some(v) = is_trivial_assign(&n) {
                        variable_substitutions.push((n.variable, v));
                        block.variable_declarations.remove(&n.variable);
                        *instr = ir::Instruction::Noop;
                    }
                }
                _ => (),
            }
        }
    }
    for var_sub in variable_substitutions {
        substitute_variable(irfunction, var_sub.0, var_sub.1);
    }
}

pub fn process_ir(mut irprogram: ir::Program) -> ir::Program {
    for irfunction in irprogram.functionstore.values_mut() {
        process_function(irfunction);
    }

    irprogram
}
