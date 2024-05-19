TARGET_DEVICE ?= pizza

.PHONY: build
build:
	cargo build --release
	cargo deb --no-build

.PHONE: install
install: build
	sudo dpkg -i PATH

.PHONY: install-dev-dependencies
install-dev-dependencies:
	cargo install cargo-deb

.PHONY: build-docker
build-docker:
	rm -rf docker_out
	mkdir docker_out
	DOCKER_BUILDKIT=1 docker build --tag robot-head-service-builder --file Dockerfile --output type=local,dest=docker_out .

.PHONY: push-docker-built
push-docker-built: build-docker
	rsync -avz --delete docker_out/servo_test $(TARGET_DEVICE):/home/dweis/servo_test

.PHONY: deploy-with-ez-cd
deploy-with-ez-cd: build-docker
	ez-cd-cli -f docker_out/robot-head-service.deb -d $(TARGET_DEVICE)
