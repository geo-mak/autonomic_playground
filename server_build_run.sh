#!/bin/bash
set -e

echo "Building docker image 'api_server'..."
docker build -t api_server .

echo "Starting the api server using compose file run-server.yml"
docker compose -f run-server.yml up --remove-orphans