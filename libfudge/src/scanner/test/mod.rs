use super::scanner::*;
use super::token::*;

mod utils;
use utils::*;

mod arithmetics;
mod basic;
mod brackets;
mod characterliterals;
mod comments;
mod identifiers;
mod indentation;
mod keywords;
mod misctokens;
mod numericliterals;
mod stringliterals;

#[test]
fn test_file_with_comments() {
    let scanner_result = get_scanner_result_from_file("testdata/comments.fu");
    // Needs to be called here directly to retain snaphot file name
    insta::assert_debug_snapshot!(&scanner_result.tokens);
}
