# Checks that the project compiles properly (does a full build)

# ===== COMPILE =====

cargo build --workspace --quiet --all-features

# ===== TESTS =====

cargo test --workspace --quiet --all-targets