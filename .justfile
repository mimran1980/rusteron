# check formatting and clippy
check:
  cargo check
  cargo fmt --all -- --check
  cargo clippy --all-targets --all-features --all -- --deny warnings
  cargo clippy --all --features static -- --deny warnings

# automatically fromat and fix simple clippy warnings
fix:
  cargo fmt --all
  cargo clippy --allow-dirty --allow-staged --fix

# clean project
clean:
  cargo clean

# build project
build:
  cargo build --all-targets

# build project with release profile
release:
  cargo build --all-targets --release

# run unit tests
test:
  cargo test --all-targets --all-features