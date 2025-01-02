# List all available tasks
list:
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