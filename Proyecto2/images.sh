#!/bin/bash

# Remove all Docker images
sudo docker rmi -f $(sudo docker images -a -q)

# Variables for the Docker images
GO_CLIENT_IMAGE="golang-client-grpc"
RUST_CLIENT_IMAGE="rust-client-grpc"
GO_SERVER_IMAGE="golang-server-grpc"
DOCKERHUB_USERNAME="brandont2002"
TAG="0.1"

# Build the Docker image for the Go client
sudo docker build -t $GO_CLIENT_IMAGE ./Agronomia
# Build the Docker image for the Rust client
sudo docker build -t $RUST_CLIENT_IMAGE ./Ingenieria
# Build the Docker image for the Go server
sudo docker build -t $GO_SERVER_IMAGE ./Server


# Tag the Docker image
sudo docker tag $GO_CLIENT_IMAGE "$DOCKERHUB_USERNAME/$GO_CLIENT_IMAGE:$TAG"
sudo docker tag $RUST_CLIENT_IMAGE "$DOCKERHUB_USERNAME/$RUST_CLIENT_IMAGE:$TAG"
sudo docker tag $GO_SERVER_IMAGE "$DOCKERHUB_USERNAME/$GO_SERVER_IMAGE:$TAG"

# Push the Docker image to DockerHub
sudo docker push "$DOCKERHUB_USERNAME/$GO_CLIENT_IMAGE:$TAG"
sudo docker push "$DOCKERHUB_USERNAME/$RUST_CLIENT_IMAGE:$TAG"
sudo docker push "$DOCKERHUB_USERNAME/$GO_SERVER_IMAGE:$TAG"

echo "Docker images pushed successfully."