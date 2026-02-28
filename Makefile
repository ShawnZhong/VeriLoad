ifeq ($(filter -j%,$(MAKEFLAGS)),)
MAKEFLAGS += -j$(shell nproc)
endif

.PHONY: all
all: veriload musl tests

BUILD_DIR := build

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

.PHONY: clean
clean:
	rm -rf $(BUILD_DIR)

# ==================== Veriload ====================
VERUS_BIN := .verus/verus-x86-linux/verus
VERILOAD_MAIN := src/main_impl.rs
VERILOAD_BINARY := $(BUILD_DIR)/veriload

.PHONY: veriload
veriload: $(VERILOAD_BINARY)
$(VERILOAD_BINARY): $(wildcard src/*.rs) | $(BUILD_DIR)
	MAKEFLAGS= $(VERUS_BIN) --compile $(VERILOAD_MAIN) -- -C target-feature=+crt-static -o $(VERILOAD_BINARY)

.PHONY: verify
verify:
	MAKEFLAGS= $(VERUS_BIN) $(VERILOAD_MAIN)

# ==================== Musl ====================
MUSL_DIR := third_party/musl
MUSL_CC := $(BUILD_DIR)/bin/musl-gcc

.PHONY: musl
musl: $(MUSL_CC)
$(MUSL_CC): | $(BUILD_DIR)
	cd $(MUSL_DIR) && ./configure --prefix=$(abspath $(BUILD_DIR))
	$(MAKE) -C $(MUSL_DIR) LDSO_OBJS= install
	ln -sfn lib/libc.so $(BUILD_DIR)/libc.so
	ln -sfn lib/libc.so $(BUILD_DIR)/libc.musl-x86_64.so.1

# ==================== Tests ====================
RELR_LDFLAGS := -Wl,-z,pack-relative-relocs

.PHONY: tests
tests: $(BUILD_DIR)/main
$(BUILD_DIR)/main: $(wildcard tests/*.c tests/*.h) $(MUSL_CC) | $(BUILD_DIR)
	$(MUSL_CC) $(RELR_LDFLAGS) -fPIC -shared -Wl,-soname,libfoo.so -o $(BUILD_DIR)/libfoo.so tests/libfoo.c
	$(MUSL_CC) $(RELR_LDFLAGS) -fPIC -shared -Wl,-soname,libbar.so -o $(BUILD_DIR)/libbar.bootstrap.so tests/libbar.c
	$(MUSL_CC) $(RELR_LDFLAGS) -fPIC -shared -Wl,-soname,libbaz.so tests/libbaz.c $(BUILD_DIR)/libbar.bootstrap.so -o $(BUILD_DIR)/libbaz.so
	$(MUSL_CC) $(RELR_LDFLAGS) -fPIC -shared -Wl,-soname,libbar.so tests/libbar.c -L$(BUILD_DIR) -lbaz -o $(BUILD_DIR)/libbar.so
	$(MUSL_CC) $(RELR_LDFLAGS) -fPIC -shared -Wl,-soname,libunused.so -o $(BUILD_DIR)/libunused.so tests/libunused.c
	$(MUSL_CC) $(RELR_LDFLAGS) tests/main.c -pthread -L$(BUILD_DIR) -lfoo -lbar -Wl,-rpath-link,$(BUILD_DIR) -o $(BUILD_DIR)/main
