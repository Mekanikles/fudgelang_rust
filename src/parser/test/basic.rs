use super::utils::*;

#[test]
fn test_empty_modulefragment() {
    verify_ast("", &module_fragment_wrapper_tree(&[]));
}
