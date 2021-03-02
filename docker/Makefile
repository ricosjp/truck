REGISTORY  := registry.ritc.jp/ricos/truck/truck
CI_CONTAINER  := rust-vulkan
IMAGE_NAME := nvidia/vulkan

CI_COMMIT_REF_SLUG ?= _no_branch


all: build

login:
ifeq ($(CI_BUILD_TOKEN),)
	docker login $(REGISTORY)
else
	docker login -u gitlab-ci-token -p $(CI_BUILD_TOKEN) $(REGISTORY)
endif

build:
	docker build -t $(REGISTORY)/rust-vulkan:$(CI_COMMIT_REF_SLUG) \
		-f build.Dockerfile \
		--target=rust-vulkan \
		--no-cache \
		.

push: login build
	docker push $(REGISTORY)/rust-vulkan:$(CI_COMMIT_REF_SLUG)

