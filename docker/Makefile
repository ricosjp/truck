REGISTORY  := registry.ritc.jp/ricos/truck/truck
CI_CONTAINER  := rust-vulkan
IMAGE_NAME := nvidia/vulkan

CI_COMMIT_REF_SLUG ?= _no_branch


all: build-rust-vulkan

login:
ifeq ($(CI_BUILD_TOKEN),)
	docker login $(REGISTORY)
else
	docker login -u gitlab-ci-token -p $(CI_BUILD_TOKEN) $(REGISTORY)
endif

build-rust-vulkan:
	docker build -t $(REGISTORY)/rust-vulkan:$(CI_COMMIT_REF_SLUG) \
		-f rust-vulkan.Dockerfile \
		--target=rust-vulkan \
		--no-cache \
		.

push-rust-vulkan: login build-rust-vulkan
	docker push $(REGISTORY)/rust-vulkan:$(CI_COMMIT_REF_SLUG)

build-rust-webtools:
	docker build -t $(REGISTORY)/rust-webtools:$(CI_COMMIT_REF_SLUG) \
		-f rust-webtools.Dockerfile \
		--target=rust-webtools \
		--no-cache \
		.

push-rust-webtools: login build-rust-webtools
	docker push $(REGISTORY)/rust-webtools:$(CI_COMMIT_REF_SLUG)
