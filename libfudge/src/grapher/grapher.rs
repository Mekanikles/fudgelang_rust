use std::collections::HashMap;

use crate::asg;
use crate::ast;
use crate::error;

struct Grapher {
    errors: error::ErrorManager,
}

pub struct GrapherResult {
    pub asg: asg::Asg,
    pub errors: Vec<error::Error>,
}

impl Grapher {
    pub fn new() -> Self {
        Grapher {
            errors: error::ErrorManager::new(),
        }
    }
}

pub fn create_graph<'a>(_main_ast: &'a ast::Ast, _module_asts: &'a Vec<ast::Ast>) -> GrapherResult {
    let grapher = Grapher::new();

    return GrapherResult {
        asg: asg::Asg {
            modules: HashMap::new(),
            main: asg::Main {
                body: asg::StatementBody {
                    statements: Vec::new(),
                    expressions: Vec::new(),
                },
            },
        },
        errors: grapher.errors.error_data.errors,
    };
}
