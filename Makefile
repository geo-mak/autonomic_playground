IMAGE_NAME = api_server
COMPOSE_FILE = run-server.yml

.PHONY: up build run

up: build run

build:
	@echo "Building docker image 'api_server'..."
	docker build -t $(IMAGE_NAME) .

run:
	@echo "Starting the api server using compose file run-server.yml..."
	docker compose -f $(COMPOSE_FILE) up --remove-orphans