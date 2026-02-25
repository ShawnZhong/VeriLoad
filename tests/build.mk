ifneq ($(strip $(shell command -v x86_64-alpine-linux-musl-gcc 2>/dev/null)),)
CC := x86_64-alpine-linux-musl-gcc
else ifneq ($(strip $(shell command -v x86_64-linux-musl-gcc 2>/dev/null)),)
CC := x86_64-linux-musl-gcc
else
$(error neither x86_64-alpine-linux-musl-gcc nor x86_64-linux-musl-gcc found in PATH)
endif

CC_STAMP := $(BUILD_DIR)/cc.stamp

.PHONY: force_cc_check
force_cc_check:

$(CC_STAMP): force_cc_check | $(BUILD_DIR)
	@if [ ! -f "$@" ] || [ "$$(cat "$@")" != "$(CC)" ]; then \
		printf '%s\n' "$(CC)" > "$@"; \
	fi

.PHONY: tests
tests: $(BUILD_DIR)/main

$(BUILD_DIR)/main: $(wildcard tests/*.c tests/*.h) $(CC_STAMP) | $(BUILD_DIR)
	$(CC) -fPIC -shared -Wl,-soname,libc_shim.so -o $(BUILD_DIR)/libc_shim.so tests/libc_shim.c
	$(CC) -fPIC -shared -Wl,-soname,libfoo.so -o $(BUILD_DIR)/libfoo.so tests/libfoo.c
	$(CC) -fPIC -shared -Wl,-soname,libbar.so -o $(BUILD_DIR)/libbar.bootstrap.so tests/libbar.c
	$(CC) -fPIC -shared -Wl,-soname,libbaz.so tests/libbaz.c $(BUILD_DIR)/libbar.bootstrap.so -o $(BUILD_DIR)/libbaz.so
	$(CC) -fPIC -shared -Wl,-soname,libbar.so tests/libbar.c -L$(BUILD_DIR) -lbaz -o $(BUILD_DIR)/libbar.so
	$(CC) -fPIC -shared -Wl,-soname,libunused.so -o $(BUILD_DIR)/libunused.so tests/libunused.c
	$(CC) tests/main.c -L$(BUILD_DIR) -lc_shim -lfoo -lbar -Wl,-rpath-link,$(BUILD_DIR) -o $(BUILD_DIR)/main
