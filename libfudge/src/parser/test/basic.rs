use super::utils::*;

#[test]
fn test_empty_modulefragment() {
    verify_ast("", &entrypoint_wrapper_tree(&[]));
}
