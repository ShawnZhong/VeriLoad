MUSL_DIR := third_party/musl
MUSL_PATCH := $(abspath third_party/musl.patch)

.PHONY: musl
musl: $(BUILD_DIR)/libc.so

$(BUILD_DIR)/libc.so: $(MUSL_PATCH) | $(BUILD_DIR)
	@set -eu; \
	if git -C $(MUSL_DIR) apply --check $(MUSL_PATCH) >/dev/null 2>&1; then \
		git -C $(MUSL_DIR) apply $(MUSL_PATCH); \
	elif git -C $(MUSL_DIR) apply -R --check $(MUSL_PATCH) >/dev/null 2>&1; then \
		:; \
	else \
		echo "failed to apply $(MUSL_PATCH)" >&2; \
		exit 1; \
	fi
	cd $(MUSL_DIR) && ./configure --prefix=$(abspath $(BUILD_DIR))
	$(MAKE) -C $(MUSL_DIR)
	$(MAKE) -C $(MUSL_DIR) install
	cp $(MUSL_DIR)/lib/libc.so $(BUILD_DIR)/libc.musl-x86_64.so.1
	ln -sfn libc.musl-x86_64.so.1 $(BUILD_DIR)/libc.so
