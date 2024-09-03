.PHONY: build
build:
	docker buildx build --tag diva-worker .

.PHONY: run
run:
	docker run --rm -it --env-file .env diva-worker
