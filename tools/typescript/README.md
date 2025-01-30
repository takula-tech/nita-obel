# How to fix release-plz github action failure

Failure 1: some crate cargo.toml not found in github action but actually locally cargo build all passed.
Solution:
modify all crates version to next version. eg, v0.0.5 -> 0.0.6
then push to main branch and wait for release-plz to finish.
