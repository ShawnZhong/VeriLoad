VERUS := ./.verus/verus-x86-linux/verus
SOURCE := src/main_impl.rs
BINARY := veriload
IMAGE_NAME := veriload

.PHONY: all
all: build

.PHONY: verify
verify:
	$(VERUS) $(SOURCE)

.PHONY: build
build:
	$(VERUS) --compile $(SOURCE) -- -C target-feature=+crt-static -o $(BINARY)

.PHONY: clean
clean:
	rm -f $(BINARY)
