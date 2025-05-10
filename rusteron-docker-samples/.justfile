# List all available tasks
list:
    cargo update
    just --list

# Automatically format code and fix simple Clippy warnings
fix:
  cd rusteron-dummy-example && cargo fmt --all && cd -
  cd rusteron-dummy-example && cargo clippy --allow-dirty --allow-staged --fix && cd -

docker-build:
    docker build -t aeron-media-driver aeron-media-driver
    docker build -t rusteron-dummy-example rusteron-dummy-example

podman-build:
    podman build -t rusteron-dummy-example -f Dockerfile rusteron-dummy-example
    podman build -t aeron-media-driver -f Dockerfile aeron-media-driver

podman-deploy:
    just podman-stop || true
    podman play kube pod.yml

podman-stop:
    podman pod stop dummy-example
    podman pod rm dummy-example --force

# assumes you have k8s e.g. via podman or docker deskstop with k8s, remember to go to settings to enable k8s
k8s-deploy:
    kubectl apply -f pod.yml

# deletes dummy-example pod
k8s-clean:
    kubectl delete pod dummy-example

run-aeron-archive-driver:
    java \
      --add-opens java.base/jdk.internal.misc=ALL-UNNAMED \
      -javaagent:../rusteron-archive/aeron/aeron-agent/build/libs/aeron-agent-1.47.5.jar \
      -cp ../rusteron-archive/aeron/aeron-all/build/libs/aeron-all-1.47.5.jar:../rusteron-archive/aeron/aeron-archive/build/libs/aeron-archive-1.47.5.jar \
      -Daeron.archive.dir=target/aeron/archive \
      -Daeron.event.log=admin \
      -Daeron.event.archive.log=all \
      -Daeron.term.buffer.sparse.file=false \
      -Daeron.pre.touch.mapped.memory=true \
      -Dagrona.disable.bounds.checks=true \
      -Daeron.dir.delete.on.start=true \
      -Daeron.dir.delete.on.shutdown=true \
      -Daeron.print.configuration=true \
      -Daeron.archive.control.channel=aeron:udp?endpoint=localhost:8010 \
      -Daeron.archive.replication.channel=aeron:udp?endpoint=localhost:8011 \
      -Daeron.archive.control.response.channel=aeron:udp?endpoint=localhost:8012 \
      io.aeron.archive.ArchivingMediaDriver

