#!/usr/bin/env bash

set -eux

# Define the name of the virtual environment
VENV_NAME=".venv"

# Check if the virtual environment already exists
if [ ! -d "$VENV_NAME" ]; then
    # If it doesn't exist, create the virtual environment
    python -m venv $VENV_NAME

    # Activate the virtual environment
    . $VENV_NAME/bin/activate

    # Upgrade pip to the latest version
    pip install --upgrade pip

    # Install the required packages from the requirements.txt file
    pip install sympy==1.12.1
    pip install cairo-lang==0.13.1

    # Deactivate the virtual environment
    deactivate
else
    # If it does exist, print a message indicating that it already exists
    echo "Virtual environment $VENV_NAME already exists."
fi
