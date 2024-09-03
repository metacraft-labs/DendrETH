.PHONY: up
up: build
	docker compose up

.PHONY: down
down:
	docker compose down

.PHONY: base
base:
	docker buildx build -t diva-base images

.PHONY: rebase
rebase:
	docker buildx build --no-cache -t diva-base images

.PHONY: build
build: base
	docker compose build

.PHONY: rebuild
rebuild:
	docker compose up --build --detach --force-recreate --no-deps ${SERVICE}
