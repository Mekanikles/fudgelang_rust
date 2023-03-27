pub mod arithmetics;
pub mod basic;
pub mod modules;
pub mod structs;
pub mod utils;

pub mod graphwalkertestingharness;
pub use graphwalkertestingharness::*;
pub mod treewalkertestingharness;
pub use treewalkertestingharness::*;

pub trait InterpreterTestingHarness {
    fn load_module_source(&mut self, module_source: &str);
    fn run(&mut self, main_source: &str) -> Box<dyn InterpreterTestingResult>;
}

pub trait InterpreterTestingResult {
    fn read_symbol_as_str(&self, module: Option<&str>, global: &str) -> String;
}
