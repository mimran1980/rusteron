list:
    just --list

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

run-aeron-archive-driver:
    cd ./rusteron-client/aeron; ./gradlew :aeron-all:build; cd -
    cd ./rusteron-client/aeron; ./gradlew :aeron-agent:jar; cd -
    java -cp ./rusteron-client/aeron/aeron-all/build/libs/aeron-all-*.jar \
      -javaagent:./rusteron-client/aeron/aeron-agent/build/libs/aeron-agent-1.47.0-SNAPSHOT.jar \
      -Daeron.dir=target/aeron \
      -Daeron.archive.dir=target/aeron/archive \
      -Daeron.event.log=all \
      -Daeron.event.archive.log=all \
      -Daeron.term.buffer.sparse.file=false \
      -Daeron.pre.touch.mapped.memory=true \
      -Daeron.socket.so_sndbuf=2m \
      -Daeron.socket.so_rcvbuf=2m \
      -Daeron.rcv.initial.window.length=2m \
      -Daeron.threading.mode=DEDICATED \
      -Daeron.sender.idle.strategy=noop \
      -Daeron.receiver.idle.strategy=noop \
      -Daeron.conductor.idle.strategy=spin \
      -Dagrona.disable.bounds.checks=true \
      -Daeron.archive.control.channel=aeron:udp?endpoint=localhost:8010 \
      -Daeron.archive.replication.channel=aeron:udp?endpoint=localhost:0 \
      -Daeron.archive.control.response.channel=aeron:udp?endpoint=localhost:0 \
      io.aeron.archive.ArchivingMediaDriver

run-aeron-media-driver-java:
    cd ./rusteron-client/aeron; ./gradlew :aeron-all:build; cd -
    java -cp ./rusteron-client/aeron/aeron-all/build/libs/aeron-all-*.jar \
      -javaagent:./rusteron-client/aeron/aeron-agent/build/libs/aeron-agent-1.47.0-SNAPSHOT.jar \
      -Daeron.dir=target/aeron \
      -Dagrona.disable.bounds.checks=true \
      io.aeron.driver.MediaDriver

run-aeron-media-driver-rust:
    AERON_DIR=target/aeron \
    cargo run --release --package rusteron-media-driver --bin media_driver

bechmark-java-ipc-throughput:
    cd ./rusteron-client/aeron; ./gradlew :aeron-samples:jar; cd -
    java -cp ./rusteron-client/aeron/aeron-all/build/libs/aeron-all-1.47.0-SNAPSHOT.jar:./rusteron-client/aeron/aeron-samples/build/libs/aeron-samples-1.47.0-SNAPSHOT.jar \
      -Daeron.dir=target/aeron \
      -Dagrona.disable.bounds.checks=true \
      -Daeron.sample.messageLength=32 \
      io.aeron.samples.EmbeddedExclusiveIpcThroughput

bechmark-rust-ipc-throughput:
    AERON_DIR=target/aeron \
    cargo run --release --package rusteron-client --example embedded_exclusive_ipc_throughput


bechmark-java-embedded-ping-pong:
    cd ./rusteron-client/aeron; ./gradlew :aeron-samples:jar; cd -
    java -cp ./rusteron-client/aeron/aeron-all/build/libs/aeron-all-1.47.0-SNAPSHOT.jar:./rusteron-client/aeron/aeron-samples/build/libs/aeron-samples-1.47.0-SNAPSHOT.jar \
      -Daeron.dir=target/aeron \
      -Dagrona.disable.bounds.checks=true \
      -Daeron.sample.exclusive.publications=false \
      -Daeron.sample.ping.channel=aeron:udp?endpoint=localhost:20123 \
      -Daeron.sample.pong.channel=aeron:udp?endpoint=localhost:20124 \
      io.aeron.samples.EmbeddedPingPong

bechmark-rust-embedded-ping-pong:
    AERON_DIR=target/aeron \
    cargo run --release --package rusteron-client --example embedded_ping_pong

bechmark-rust-embedded-ping-pong-profiler:
    cargo build --features static --release --package rusteron-client --example embedded_ping_pong
    codesign -s - -vvv --entitlements instruments.plist ./target/release/examples/embedded_ping_pong
    AERON_DIR=target/aeron \
    ./target/release/examples/embedded_ping_pong

bechmarks:
    cargo bench

docs:
  cargo clean --doc
  cargo test --doc
  cargo doc --workspace --no-deps --open

# run unit tests
test:
  cargo test --all-targets --all-features -- --nocapture