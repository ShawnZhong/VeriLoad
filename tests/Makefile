CC ?= cc
BUILD_DIR ?= build
NOLIBC_DIR ?= ../third_party/nolibc

COMMON_CFLAGS := \
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

COMMON_LDFLAGS := \
	-Wl,--gc-sections

LIBS := \
	$(BUILD_DIR)/libfoo.so \
	$(BUILD_DIR)/libbar.so \
	$(BUILD_DIR)/libbaz.so

BIN := $(BUILD_DIR)/main

.PHONY: all
all: $(LIBS) $(BIN)

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

$(BUILD_DIR)/libbaz.so: libbaz.c libbaz.h | $(BUILD_DIR)
	$(CC) $(COMMON_CFLAGS) -fPIC -shared $(COMMON_LDFLAGS) -Wl,-soname,libbaz.so -o $@ $<

$(BUILD_DIR)/libbar.so: libbar.c libbar.h libbaz.h $(BUILD_DIR)/libbaz.so | $(BUILD_DIR)
	$(CC) $(COMMON_CFLAGS) -fPIC -shared $(COMMON_LDFLAGS) -Wl,-soname,libbar.so $< -L$(BUILD_DIR) -lbaz -o $@

$(BUILD_DIR)/libfoo.so: libfoo.c libfoo.h | $(BUILD_DIR)
	$(CC) $(COMMON_CFLAGS) -fPIC -shared $(COMMON_LDFLAGS) -Wl,-soname,libfoo.so -o $@ $<

$(BUILD_DIR)/main: main.c libfoo.h libbar.h $(BUILD_DIR)/libfoo.so $(BUILD_DIR)/libbar.so | $(BUILD_DIR)
	$(CC) $(COMMON_CFLAGS) -no-pie -Wl,-e,_start $(COMMON_LDFLAGS) $< -L$(BUILD_DIR) -lfoo -lbar -Wl,-rpath-link,$(BUILD_DIR) -o $@

.PHONY: clean
clean:
	rm -rf $(BUILD_DIR)
