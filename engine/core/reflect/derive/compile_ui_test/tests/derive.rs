//! contains the test suite for verifying the behavior of derive macros
//! in the reflect system. It runs compile-time tests to ensure proper error messages
//! and derivation behavior.

fn main() -> obel_ui_test_runner::ui_test::Result<()> {
    obel_ui_test_runner::test_multiple(
        "derive_deref",
        ["tests/deref_derive", "tests/deref_mut_derive"],
    )
}
