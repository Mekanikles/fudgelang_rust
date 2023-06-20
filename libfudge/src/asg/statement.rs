use super::*;

use scope::ExpressionKey;

pub mod statements {
    use super::*;

    #[derive(Debug)]
    pub struct Branch {
        pub scope: ScopeKey,
        pub body: Option<StatementBody>,
    }

    #[derive(Debug)]
    pub struct If {
        pub branches: Vec<(ExpressionKey, Branch)>,
        pub elsebranch: Option<Branch>,
    }

    #[derive(Debug)]
    pub struct Return {
        pub expr: Option<ExpressionKey>,
    }

    #[derive(Debug)]
    pub struct Assign {
        pub lhs: ExpressionKey,
        pub rhs: ExpressionKey,
    }

    #[derive(Debug)]
    pub struct Initialize {
        pub symbol: String,
        pub expr: ExpressionKey,
    }

    #[derive(Debug)]
    pub struct ExpressionWrapper {
        pub expr: ExpressionKey,
    }
}

#[derive(Debug)]
pub enum Statement {
    If(statements::If),
    Return(statements::Return),
    Initialize(statements::Initialize),
    Assign(statements::Assign),
    ExpressionWrapper(statements::ExpressionWrapper),
}
