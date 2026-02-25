TESTS_DIR ?= tests
CC := musl-gcc

.PHONY: tests
tests: \
	$(BUILD_DIR)/libc_shim.so \
	$(BUILD_DIR)/libfoo.so \
	$(BUILD_DIR)/libbar.so \
	$(BUILD_DIR)/libbaz.so \
	$(BUILD_DIR)/libunused.so \
	$(BUILD_DIR)/main

$(BUILD_DIR)/libbar.bootstrap.so: $(TESTS_DIR)/libbar.c $(TESTS_DIR)/libbar.h $(TESTS_DIR)/libbaz.h | $(BUILD_DIR)
	$(CC) -fPIC -shared -Wl,-soname,libbar.so -o $@ $<

$(BUILD_DIR)/libbaz.so: $(TESTS_DIR)/libbaz.c $(TESTS_DIR)/libbaz.h $(TESTS_DIR)/libbar.h $(BUILD_DIR)/libbar.bootstrap.so | $(BUILD_DIR)
	$(CC) -fPIC -shared -Wl,-soname,libbaz.so $< $(BUILD_DIR)/libbar.bootstrap.so -o $@

$(BUILD_DIR)/libbar.so: $(TESTS_DIR)/libbar.c $(TESTS_DIR)/libbar.h $(TESTS_DIR)/libbaz.h $(BUILD_DIR)/libbaz.so | $(BUILD_DIR)
	$(CC) -fPIC -shared -Wl,-soname,libbar.so $< -L$(BUILD_DIR) -lbaz -o $@

$(BUILD_DIR)/libfoo.so: $(TESTS_DIR)/libfoo.c $(TESTS_DIR)/libfoo.h | $(BUILD_DIR)
	$(CC) -fPIC -shared -Wl,-soname,libfoo.so -o $@ $<

$(BUILD_DIR)/libunused.so: $(TESTS_DIR)/libunused.c | $(BUILD_DIR)
	$(CC) -fPIC -shared -Wl,-soname,libunused.so -o $@ $<

$(BUILD_DIR)/libc_shim.so: $(TESTS_DIR)/libc_shim.c | $(BUILD_DIR)
	$(CC) -fPIC -shared -Wl,-soname,libc_shim.so -o $@ $<

$(BUILD_DIR)/main: $(TESTS_DIR)/main.c $(TESTS_DIR)/libfoo.h $(TESTS_DIR)/libbar.h $(BUILD_DIR)/libc_shim.so $(BUILD_DIR)/libfoo.so $(BUILD_DIR)/libbar.so | $(BUILD_DIR)
	$(CC) $< -L$(BUILD_DIR) -lc_shim -lfoo -lbar -Wl,-rpath-link,$(BUILD_DIR) -o $@
