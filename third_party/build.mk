MUSL_DIR := third_party/musl
MUSL_CC := $(BUILD_DIR)/bin/musl-gcc

.PHONY: musl
musl: $(MUSL_CC)

$(MUSL_CC): | $(BUILD_DIR)
	cd $(MUSL_DIR) && ./configure --prefix=$(abspath $(BUILD_DIR))
	$(MAKE) -C $(MUSL_DIR) LDSO_OBJS= install
	ln -sfn lib/libc.so $(BUILD_DIR)/libc.so
	ln -sfn lib/libc.so $(BUILD_DIR)/libc.musl-x86_64.so.1
