// Test on the output of the current binary when parsing.
// This will only work with the `test` binary.
const TEST_BIN: &'static str = "test";

// Different constants which might change if the
// error messages's contents  ever change (they likely will)
// Most of these could be taken from `src/parse.rs` but by
// copying them here instead, the following tests can
// stand fully on their own.
const KEY_NAME: &'static str = "key";
const CONSTANT_NAME: &'static str = "constant";
const OPTION_NAME: &'static str = "option";
const CTX_CONTAINS: &'static str = "Found invalid character(s) contained in ";
const CTX_OPENS: &'static str = "Found invalid opening character for ";
const CTX_CLOSES: &'static str = "Found invalid closing character for ";
const PSBL_MAYBE: &'static str = "Did you maybe mean ";
const PSBL_ALLOWED: &'static str = "Allowed characters are ";
const PSBL_FORBIDDEN: &'static str = "Forbidden characters are ";

// Make assertions on the behaviour of the binary
use assert_cmd::Command;
use self::helper::assert_out;

#[test]
fn invalid_key_ident() {
    // The ident of the key is interrupted by an invalid character
    assert_out("{nam*e}", vec![CTX_CLOSES, PSBL_MAYBE]);  // TODO: Update this to CTX_CONTAINS
}

mod helper {
    use super::*;

    // Make assertions on the output of the binary
    // given an input
    pub fn assert_out(input: &str, contained: Vec<&str>) {
        for output in contained {
            Command::cargo_bin(TEST_BIN)
                .unwrap()
                .write_stdin(input)
                .assert()
                .stderr(predicates::str::contains(output));
        }
    }
}
