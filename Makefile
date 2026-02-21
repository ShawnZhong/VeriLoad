VERUS := ./.verus/verus-x86-linux/verus
SOURCE := src/main.rs
BINARY := veriload

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
	podman build -q -t veriload -f Containerfile .
