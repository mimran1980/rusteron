# Dummy Example for Docker and Kubernetes Configuration

This repository provides a simple example to demonstrate containerization and orchestration using **Docker**, **Podman**, and **Kubernetes**. While not intended as production-ready, it serves as a helpful starting point.

## Overview

The setup includes:

1. **Aeron Media Driver**: A containerized Aeron Archive Media Driver.
2. **Ticker Writer**: Publishes and archives Binance ticker JSON messages.
3. **Ticker Reader**: Periodically publishes stats about the archive and live ticker channel.

## Quick Start for Local Testing

For local testing, Podman is recommended for its lightweight nature and ease of use. If you have a Kubernetes cluster, you can deploy using Kubernetes configurations.

### Podman Local Testing

1. **Build the Podman Images**  
   Build all required images using Podman:
   ```bash
   just podman-build
   ```

2. **Deploy Using Podman**  
   Deploy the example configuration with:
   ```bash
   just podman-deploy
   ```

3. **Stop and Clean Up Podman Resources**  
   To stop the deployment:
   ```bash
   just podman-stop
   ```

### Kubernetes Deployment (Alternative)

If you have a Kubernetes cluster, you can use it for deployment:

1. **Build Docker Images**  
   Ensure the images are built:
   ```bash
   just docker-build
   ```

2. **Deploy to Kubernetes**  
   Apply the pod configuration to your Kubernetes cluster:
   ```bash
   just k8s-deploy
   ```

3. **Clean Up Kubernetes Resources**  
   Remove the deployed pod:
   ```bash
   just k8s-clean
   ```

## Prerequisites

- **Podman** or **Docker** for container management.
- **Kubernetes** (optional) for orchestration. Ensure it's enabled if using Docker Desktop.
- **just** for task automation. Install `just` from its [GitHub repository](https://github.com/casey/just).

## Key Features Demonstrated

- Shared memory (`/dev/shm`) and data (`/data`) volumes for container interaction.
- Configurable arguments and shared environments using `pod.yml`.
- Task automation for building, deploying, and cleaning with `just`.

> Note: This example is designed for experimentation and learning. For production, additional configuration, security, and testing are required.