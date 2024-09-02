#!/usr/bin/env bash

set -eux

IMAGE_NAME="localhost/http-prover-test"

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

KEYGEN_OUTPUT=$(cargo run -p keygen)

PUBLIC_KEY=$(echo "$KEYGEN_OUTPUT" | grep "Public key" | awk '{print $3}' | tr -d ',' | tr -d '[:space:]')
PRIVATE_KEY=$(echo "$KEYGEN_OUTPUT" | grep "Private key" | awk '{print $3}' | tr -d ',' | tr -d '[:space:]')

echo "Public Key: $PUBLIC_KEY"
echo "Private Key: $PRIVATE_KEY"

podman run -d --replace --name http_prover_test \
    -p 3040:3000 $IMAGE_NAME \
    --jwt-secret-key "secret" \
    --message-expiration-time 3600 \
    --session-expiration-time 3600 \
    --authorized-keys $PUBLIC_KEY \
    --admin-key 0xd16b71c90dbf897e5964d2f267d04664b3c035036559d712994739ea6cf2fd9f

PRIVATE_KEY=$PRIVATE_KEY PROVER_URL="http://localhost:3040" cargo test --no-fail-fast --workspace

podman stop http_prover_test
