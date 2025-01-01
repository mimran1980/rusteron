# List all available tasks
list:
    just --list

build:
    docker build -t aeron-media-driver aeron-media-driver
    docker build -t rusteron-dummy-example rusteron-dummy-example

# assumes your using docker deskstop with k8s, remember to go to settings to enable k8s
deploy:
    kubectl apply -f pod.yml

# deletes dummy-example pod
clean:
    kubectl delete pod dummy-example