# List all available tasks
list:
    cargo install cargo-udeps
    cargo install cargo-edit
    cargo update
    just --list

# Check formatting, linting, and compile
check:
  cargo check --workspace
  cargo fmt --all -- --check
  cargo clippy --all-targets --all-features --all -- --deny warnings
  cargo clippy -- --deny warnings

# Automatically format code and fix simple Clippy warnings
fix:
  cargo fmt --all
  cargo clippy --allow-dirty --allow-staged --fix -- -W unused_imports -W clippy::all
  cd rusteron-docker-samples/rusteron-dummy-example && just fix

check-udeps:
  cargo +nightly tree --duplicate
  cargo +nightly udeps


# Clean the project by removing the target directory
clean:
  rm -rf rusteron-archive/target
  rm -rf rusteron-client/target
  rm -rf rusteron-media-driver/target
  rm -rf rusteron-rb/target
  rm -rf rusteron-docker-samples/target
  rm -rf rusteron-docker-samples/rusteron-dummy-example/target
  cd rusteron-archive/aeron && git submodule update --init --recursive --checkout && git reset --hard && git clean -fdx && cd -
  cd rusteron-client/aeron && git submodule update --init --recursive --checkout && git reset --hard && git clean -fdx && cd -
  cd rusteron-media-driver/aeron && git submodule update --init --recursive --checkout && git reset --hard && git clean -fdx && cd -
  cd rusteron-rb/aeron && git submodule update --init --recursive --checkout && git reset --hard && git clean -fdx && cd -
  cargo clean

# Build the project in debug mode
build:
  COPY_BINDINGS=true cargo build --workspace --all-targets

# Build the project in release mode
release:
  cargo build --workspace --all-targets --release

# Run the Aeron archive driver using Java
run-aeron-archive-driver:
    cd ./rusteron-client/aeron; ./gradlew :aeron-all:build; cd -
    cd ./rusteron-client/aeron; ./gradlew :aeron-agent:jar; cd -
    java \
      --add-opens java.base/jdk.internal.misc=ALL-UNNAMED \
      -javaagent:./rusteron-client/aeron/aeron-agent/build/libs/aeron-agent-1.48.0-SNAPSHOT.jar \
      -cp ./rusteron-client/aeron/aeron-all/build/libs/aeron-all-1.48.0-SNAPSHOT.jar:./rusteron-client/aeron/aeron-archive/build/libs/aeron-archive-1.48.0-SNAPSHOT.jar \
      -Daeron.dir=target/aeron \
      -Daeron.archive.dir=target/aeron/archive \
      -Daeron.event.log=all \
      -Daeron.event.archive.log=all \
      -Daeron.term.buffer.sparse.file=false \
      -Daeron.pre.touch.mapped.memory=true \
      -Daeron.threading.mode=DEDICATED \
      -Daeron.sender.idle.strategy=noop \
      -Daeron.receiver.idle.strategy=noop \
      -Daeron.conductor.idle.strategy=spin \
      -Dagrona.disable.bounds.checks=true \
      -Daeron.dir.delete.on.start=true \
      -Daeron.dir.delete.on.shutdown=true \
      -Daeron.print.configuration=true \
      -Daeron.archive.control.channel=aeron:udp?endpoint=localhost:8010 \
      -Daeron.archive.replication.channel=aeron:udp?endpoint=localhost:0 \
      -Daeron.archive.control.response.channel=aeron:udp?endpoint=localhost:0 \
      io.aeron.archive.ArchivingMediaDriver

# Run the Aeron Media Driver using Java
run-aeron-media-driver-java:
    cd ./rusteron-client/aeron; ./gradlew :aeron-all:build; cd -
    java -cp ./rusteron-client/aeron/aeron-all/build/libs/aeron-all-*.jar \
      --add-opens java.base/jdk.internal.misc=ALL-UNNAMED \
      -Daeron.dir=target/aeron \
      -Daeron.term.buffer.sparse.file=false \
      -Daeron.pre.touch.mapped.memory=true \
      -Daeron.threading.mode=DEDICATED \
      -Daeron.sender.idle.strategy=noop \
      -Daeron.receiver.idle.strategy=noop \
      -Daeron.conductor.idle.strategy=spin \
      -Dagrona.disable.bounds.checks=true \
      -Daeron.dir.delete.on.start=true \
      -Daeron.dir.delete.on.shutdown=true \
      -Daeron.print.configuration=true \
      io.aeron.driver.MediaDriver

# Run the Aeron Media Driver using Rust
run-aeron-media-driver-rust:
    AERON_DIR_DELETE_ON_START=true \
    AERON_DIR_DELETE_ON_SHUTDOWN=true \
    AERON_PRINT_CONFIGURATION=true \
    AERON_THREADING_MODE=DEDICATED \
    AERON_CONDUCTOR_IDLE_STRATEGY=spin \
    AERON_SENDER_IDLE_STRATEGY=noop \
    AERON_RECEIVER_IDLE_STRATEGY=noop \
    AERON_DIR=target/aeron \
    AERON_TERM_BUFFER_SPARSE_FILE=false \
    cargo run --release --package rusteron-media-driver --bin media_driver

benchmark-ipc-throughput-java:
    cd ./rusteron-client/aeron; ./gradlew :aeron-all:build; cd -
    cd ./rusteron-client/aeron; ./gradlew :aeron-samples:jar; cd -
    java -cp ./rusteron-client/aeron/aeron-all/build/libs/aeron-all-1.48.0-SNAPSHOT.jar:./rusteron-client/aeron/aeron-samples/build/libs/aeron-samples-1.48.0-SNAPSHOT.jar \
      --add-opens java.base/jdk.internal.misc=ALL-UNNAMED \
      -Daeron.dir=target/aeron \
      -Daeron.term.buffer.sparse.file=false \
      -Daeron.pre.touch.mapped.memory=true \
      -Daeron.threading.mode=DEDICATED \
      -Daeron.sender.idle.strategy=noop \
      -Daeron.receiver.idle.strategy=noop \
      -Daeron.conductor.idle.strategy=spin \
      -Dagrona.disable.bounds.checks=true \
      -Daeron.dir.delete.on.start=true \
      -Daeron.dir.delete.on.shutdown=true \
      -Daeron.print.configuration=true \
      -Daeron.sample.messageLength=32 \
      -Daeron.sample.idleStrategy=org.agrona.concurrent.NoOpIdleStrategy \
      io.aeron.samples.EmbeddedExclusiveIpcThroughput

benchmark-ipc-throughput-rust:
    AERON_DIR_DELETE_ON_START=true \
    AERON_DIR_DELETE_ON_SHUTDOWN=true \
    AERON_PRINT_CONFIGURATION=true \
    AERON_THREADING_MODE=SHARED \
    AERON_CONDUCTOR_IDLE_STRATEGY=spin \
    AERON_SENDER_IDLE_STRATEGY=noop \
    AERON_RECEIVER_IDLE_STRATEGY=noop \
    AERON_DIR=target/aeron \
    AERON_TERM_BUFFER_SPARSE_FILE=false \
    AERON_DIR=target/aeron \
    cargo run --release --package rusteron-client --example embedded_exclusive_ipc_throughput

benchmark-ipc-throughput-rust-profiler:
    sudo AERON_DIR_DELETE_ON_START=true \
    AERON_DIR_DELETE_ON_SHUTDOWN=true \
    AERON_PRINT_CONFIGURATION=true \
    AERON_THREADING_MODE=DEDICATED \
    AERON_CONDUCTOR_IDLE_STRATEGY=spin \
    AERON_SENDER_IDLE_STRATEGY=noop \
    AERON_RECEIVER_IDLE_STRATEGY=noop \
    AERON_DIR=target/aeron \
    AERON_TERM_BUFFER_SPARSE_FILE=false \
    LD_LIBRARY_PATH=target/release/build/rusteron-client-*/out/build/lib \
    perf record -g ./target/release/examples/embedded_exclusive_ipc_throughput
    sudo perf report

benchmark-embedded-ping-pong-java:
    cd ./rusteron-client/aeron; ./gradlew :aeron-samples:jar; cd -
    java -cp ./rusteron-client/aeron/aeron-all/build/libs/aeron-all-1.48.0-SNAPSHOT.jar:./rusteron-client/aeron/aeron-samples/build/libs/aeron-samples-1.48.0-SNAPSHOT.jar \
      --add-opens java.base/jdk.internal.misc=ALL-UNNAMED \
      -Daeron.dir=target/aeron \
      -Daeron.term.buffer.sparse.file=false \
      -Daeron.pre.touch.mapped.memory=true \
      -Daeron.threading.mode=DEDICATED \
      -Daeron.sender.idle.strategy=noop \
      -Daeron.receiver.idle.strategy=noop \
      -Daeron.conductor.idle.strategy=spin \
      -Dagrona.disable.bounds.checks=true \
      -Daeron.dir.delete.on.start=true \
      -Daeron.dir.delete.on.shutdown=true \
      -Daeron.print.configuration=true \
      -Daeron.sample.messageLength=32 \
      -Daeron.sample.idleStrategy=org.agrona.concurrent.NoOpIdleStrategy \
      -Daeron.sample.exclusive.publications=false \
      -Daeron.sample.ping.channel=aeron:udp?endpoint=localhost:20123 \
      -Daeron.sample.pong.channel=aeron:udp?endpoint=localhost:20124 \
      io.aeron.samples.EmbeddedPingPong

benchmark-embedded-ping-pong-rust:
    AERON_DIR_DELETE_ON_START=true \
    AERON_DIR_DELETE_ON_SHUTDOWN=true \
    AERON_PRINT_CONFIGURATION=true \
    AERON_THREADING_MODE=DEDICATED \
    AERON_CONDUCTOR_IDLE_STRATEGY=spin \
    AERON_SENDER_IDLE_STRATEGY=noop \
    AERON_RECEIVER_IDLE_STRATEGY=noop \
    AERON_DIR=target/aeron \
    AERON_TERM_BUFFER_SPARSE_FILE=false \
    cargo run --release --package rusteron-client --example embedded_ping_pong

benchmark-embedded-ping-pong-rust-profiler:
    cargo build --features static --release --package rusteron-client --example embedded_ping_pong
    codesign -s - -vvv --entitlements instruments.plist ./target/release/examples/embedded_ping_pong
    AERON_DIR=target/aeron \
    ./target/release/examples/embedded_ping_pong

# runs criteron benchmarks
bench:
    cargo bench

# generate rust docs locally
docs:
  cargo clean --doc
  cargo test  --workspace --doc
  cargo doc --workspace --no-deps --open

# run unit tests
test:
  cargo test --workspace -- --nocapture
  cargo test  --workspace --doc
  cargo test  --workspace --all-targets --all-features -- --nocapture

# creates symbolic link so that taret/aeron goes to /dev/shm/aeron (when benchmarking on linux)
create-sym-link:
    rm -rfv target/aeron
    mkdir -p target/aeron
    ls -l target/aeron


# e.g just aeron-archive-tool ./rusteron-archive/target/aeron/784454882946541/shm/archive describe/dump/errors
aeron-archive-tool dir action:
    java --add-opens java.base/jdk.internal.misc=ALL-UNNAMED -cp ./rusteron-client/aeron/aeron-all/build/libs/aeron-all-*.jar io.aeron.archive.ArchiveTool {{dir}} {{action}}

# e.g just aeron-archive-tool ./rusteron-archive/target/aeron/784454882946541/shm
aeron-stat dir:
    java --add-opens java.base/jdk.internal.misc=ALL-UNNAMED -cp ./rusteron-archive/aeron/aeron-all/build/libs/aeron-all-*.jar -Daeron.dir={{dir}} io.aeron.samples.AeronStat

build-docker-samples:
    cargo update
    cd rusteron-docker-samples/rusteron-dummy-example && cargo build --release && cd ..
    cd rusteron-docker-samples && just build

# updates aeron version e.g. tags/1.47.3 or master
update-aeron-version version:
    cd rusteron-client/aeron && git checkout {{version}} && cd -
    cd rusteron-archive/aeron && git checkout {{version}} && cd -
    cd rusteron-media-driver/aeron && git checkout {{version}} && cd -

update_to-latest-aeron-version:
    just update-aeron-version tags/`curl -s https://github.com/aeron-io/aeron/releases | grep -o 'tag/[0-9]*\.[0-9]*\.[0-9]*' | head -1 | cut -d'/' -f2`

