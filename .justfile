# check formatting and clippy
check:
  cargo check
  cargo fmt --all -- --check
  cargo clippy --all-targets --all-features --all -- --deny warnings
  cargo clippy -- --deny warnings

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

run-aeron-archive:
    java -cp ./rusteron-archive/aeron/aeron-all/build/libs/aeron-all-*.jar -Daeron.archive.control.channel=aeron:udp?endpoint=localhost:8010 -Daeron.archive.replication.channel=aeron:udp?endpoint=localhost:0 -Daeron.archive.control.response.channel=aeron:udp?endpoint=localhost:0

docs:
  cargo clean --doc
  cargo test --doc
  cargo doc --workspace --no-deps --open

# run unit tests
test:
  cargo test --all-targets --all-features