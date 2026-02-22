TESTS_DIR ?= tests
NOLIBC_DIR ?= $(TESTS_DIR)/../third_party/nolibc
CC ?= cc

TEST_COMMON_CFLAGS := \
	-nostdlib \
	-DNOLIBC_NO_RUNTIME \
	-DNOLIBC_IGNORE_ERRNO \
	-I$(NOLIBC_DIR)

TEST_LIBS := \
	$(BUILD_DIR)/libfoo.so \
	$(BUILD_DIR)/libbar.so \
	$(BUILD_DIR)/libbaz.so \
	$(BUILD_DIR)/libunused.so

TEST_BIN := $(BUILD_DIR)/main

.PHONY: tests
tests: $(TEST_LIBS) $(TEST_BIN)

$(BUILD_DIR)/libbar.bootstrap.so: $(TESTS_DIR)/libbar.c $(TESTS_DIR)/libbar.h $(TESTS_DIR)/libbaz.h | $(BUILD_DIR)
	$(CC) $(TEST_COMMON_CFLAGS) -fPIC -shared -Wl,-soname,libbar.so -o $@ $<

$(BUILD_DIR)/libbaz.so: $(TESTS_DIR)/libbaz.c $(TESTS_DIR)/libbaz.h $(TESTS_DIR)/libbar.h $(BUILD_DIR)/libbar.bootstrap.so | $(BUILD_DIR)
	$(CC) $(TEST_COMMON_CFLAGS) -fPIC -shared -Wl,-soname,libbaz.so $< $(BUILD_DIR)/libbar.bootstrap.so -o $@

$(BUILD_DIR)/libbar.so: $(TESTS_DIR)/libbar.c $(TESTS_DIR)/libbar.h $(TESTS_DIR)/libbaz.h $(BUILD_DIR)/libbaz.so | $(BUILD_DIR)
	$(CC) $(TEST_COMMON_CFLAGS) -fPIC -shared -Wl,-soname,libbar.so $< -L$(BUILD_DIR) -lbaz -o $@

$(BUILD_DIR)/libfoo.so: $(TESTS_DIR)/libfoo.c $(TESTS_DIR)/libfoo.h | $(BUILD_DIR)
	$(CC) $(TEST_COMMON_CFLAGS) -fPIC -shared -Wl,-soname,libfoo.so -o $@ $<

$(BUILD_DIR)/libunused.so: $(TESTS_DIR)/libunused.c | $(BUILD_DIR)
	$(CC) $(TEST_COMMON_CFLAGS) -fPIC -shared -Wl,-soname,libunused.so -o $@ $<

$(BUILD_DIR)/main: $(TESTS_DIR)/main.c $(TESTS_DIR)/libfoo.h $(TESTS_DIR)/libbar.h $(BUILD_DIR)/libfoo.so $(BUILD_DIR)/libbar.so | $(BUILD_DIR)
	$(CC) $(TEST_COMMON_CFLAGS) -no-pie -Wl,-e,main $< -L$(BUILD_DIR) -lfoo -lbar -Wl,-rpath-link,$(BUILD_DIR) -o $@
