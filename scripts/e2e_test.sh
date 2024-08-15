#!/usr/bin/env bash

IMAGE_NAME="http_prover_test"

# Check if the image already exists
if podman images | grep -q "$IMAGE_NAME"; then
    echo "Image $IMAGE_NAME already exists. Skipping build step."
else
    echo "Image $IMAGE_NAME does not exist. Building the image..."
    podman build -t $IMAGE_NAME .
    if [ $? -ne 0 ]; then
        echo "Failed to build the image. Exiting."
        exit 1
    fi
fi

podman run -d --replace --name http_prover_test \
    -p 3040:3000 localhost/http_prover_test \
    --jwt-secret-key "jwt" \
    --message-expiration-time 3600 \
    --session-expiration-time 3600 \
    --authorized-keys 0xed126082726a1062ed6e886b2d7aba42e4f8964a13b4569988bd4c50b9a62076
if [ $? -ne 0 ]; then
    echo "Failed to run the image. Exiting."
    exit 1
fi

cargo test --no-fail-fast --workspace --verbose

podman stop $IMAGE_NAME
