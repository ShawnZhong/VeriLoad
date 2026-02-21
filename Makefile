VERUS := ./.verus/verus-x86-linux/verus
SOURCE := src/main.rs
BINARY := veriload
IMAGE_NAME := veriload

.PHONY: all
all: build container

.PHONY: verify
verify:
	$(VERUS) $(SOURCE)

.PHONY: build
build:
	$(VERUS) --compile $(SOURCE) -- -C target-feature=+crt-static -o $(BINARY)

.PHONY: clean
clean:
	rm -f $(BINARY)

.PHONY: container
container: Containerfile
	podman build -q -t $(IMAGE_NAME) -f Containerfile .
