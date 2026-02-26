MUSL_CC := $(BUILD_DIR)/bin/musl-gcc

.PHONY: tests
tests: $(BUILD_DIR)/main

$(BUILD_DIR)/main: $(wildcard tests/*.c tests/*.h) musl | $(BUILD_DIR)
	$(MUSL_CC) -fPIC -shared -Wl,-soname,libfoo.so -o $(BUILD_DIR)/libfoo.so tests/libfoo.c
	$(MUSL_CC) -fPIC -shared -Wl,-soname,libbar.so -o $(BUILD_DIR)/libbar.bootstrap.so tests/libbar.c
	$(MUSL_CC) -fPIC -shared -Wl,-soname,libbaz.so tests/libbaz.c $(BUILD_DIR)/libbar.bootstrap.so -o $(BUILD_DIR)/libbaz.so
	$(MUSL_CC) -fPIC -shared -Wl,-soname,libbar.so tests/libbar.c -L$(BUILD_DIR) -lbaz -o $(BUILD_DIR)/libbar.so
	$(MUSL_CC) -fPIC -shared -Wl,-soname,libunused.so -o $(BUILD_DIR)/libunused.so tests/libunused.c
	$(MUSL_CC) tests/main.c -pthread -L$(BUILD_DIR) -lfoo -lbar -Wl,-rpath-link,$(BUILD_DIR) -o $(BUILD_DIR)/main
