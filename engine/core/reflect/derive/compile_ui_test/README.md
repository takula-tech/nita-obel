# Compile UI Test

This crate is separate from `obel_reflect` and not part of the Bevy workspace in order to not fail `crater` tests for
Obel.
The tests assert on the exact compiler errors and can easily fail for new Rust versions due to updated compiler errors (e.g. changes in spans).

The `CI` workflow executes these tests on the stable rust toolchain (see [tools/ci](../../../../tools/typescript/ci.ts)).

For information on writing tests see [ui_test_runner/README.md](../../../../tools/ui_test_runner/README.md).
