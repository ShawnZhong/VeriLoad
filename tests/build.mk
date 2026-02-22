TESTS_DIR ?= tests
NOLIBC_DIR ?= $(TESTS_DIR)/../third_party/nolibc
CC ?= cc

TEST_COMMON_CFLAGS := \
	-nostdlib \
	-ffunction-sections \
	-fdata-sections \
	-fno-asynchronous-unwind-tables \
	-fno-unwind-tables \
	-fno-stack-protector \
	-fno-ident \
	-DNOLIBC_NO_RUNTIME \
	-DNOLIBC_IGNORE_ERRNO \
	-I$(NOLIBC_DIR)

TEST_COMMON_LDFLAGS := \
	-Wl,--gc-sections

TEST_LIBS := \
	$(BUILD_DIR)/libfoo.so \
	$(BUILD_DIR)/libbar.so \
	$(BUILD_DIR)/libbaz.so \
	$(BUILD_DIR)/libunused.so

TEST_BIN := $(BUILD_DIR)/main

.PHONY: tests
tests: $(TEST_LIBS) $(TEST_BIN)

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

$(BUILD_DIR)/libbar.bootstrap.so: $(TESTS_DIR)/libbar.c $(TESTS_DIR)/libbar.h $(TESTS_DIR)/libbaz.h | $(BUILD_DIR)
	$(CC) $(TEST_COMMON_CFLAGS) -fPIC -shared $(TEST_COMMON_LDFLAGS) -Wl,-soname,libbar.so -o $@ $<

$(BUILD_DIR)/libbaz.so: $(TESTS_DIR)/libbaz.c $(TESTS_DIR)/libbaz.h $(TESTS_DIR)/libbar.h $(BUILD_DIR)/libbar.bootstrap.so | $(BUILD_DIR)
	$(CC) $(TEST_COMMON_CFLAGS) -fPIC -shared $(TEST_COMMON_LDFLAGS) -Wl,-soname,libbaz.so $< $(BUILD_DIR)/libbar.bootstrap.so -o $@

$(BUILD_DIR)/libbar.so: $(TESTS_DIR)/libbar.c $(TESTS_DIR)/libbar.h $(TESTS_DIR)/libbaz.h $(BUILD_DIR)/libbaz.so | $(BUILD_DIR)
	$(CC) $(TEST_COMMON_CFLAGS) -fPIC -shared $(TEST_COMMON_LDFLAGS) -Wl,-soname,libbar.so $< -L$(BUILD_DIR) -lbaz -o $@

$(BUILD_DIR)/libfoo.so: $(TESTS_DIR)/libfoo.c $(TESTS_DIR)/libfoo.h | $(BUILD_DIR)
	$(CC) $(TEST_COMMON_CFLAGS) -fPIC -shared $(TEST_COMMON_LDFLAGS) -Wl,-soname,libfoo.so -o $@ $<

$(BUILD_DIR)/libunused.so: $(TESTS_DIR)/libunused.c | $(BUILD_DIR)
	$(CC) $(TEST_COMMON_CFLAGS) -fPIC -shared $(TEST_COMMON_LDFLAGS) -Wl,-soname,libunused.so -o $@ $<

$(BUILD_DIR)/main: $(TESTS_DIR)/main.c $(TESTS_DIR)/libfoo.h $(TESTS_DIR)/libbar.h $(BUILD_DIR)/libfoo.so $(BUILD_DIR)/libbar.so | $(BUILD_DIR)
	$(CC) $(TEST_COMMON_CFLAGS) -no-pie -Wl,-e,main $(TEST_COMMON_LDFLAGS) $< -L$(BUILD_DIR) -lfoo -lbar -Wl,-rpath-link,$(BUILD_DIR) -o $@
