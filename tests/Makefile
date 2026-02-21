CC ?= cc
BUILD_DIR ?= build

LIBS := \
	$(BUILD_DIR)/libveriload_basic.so \
	$(BUILD_DIR)/libveriload_dep_b.so \
	$(BUILD_DIR)/libveriload_dep_a.so \
	$(BUILD_DIR)/libveriload_missing_leaf.so \
	$(BUILD_DIR)/libveriload_missing_top.so

BINS := \
	$(BUILD_DIR)/pos_no_deps \
	$(BUILD_DIR)/pos_one_dep \
	$(BUILD_DIR)/pos_transitive_dep \
	$(BUILD_DIR)/neg_missing_dep

.PHONY: all
all: $(LIBS) $(BINS)

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

$(BUILD_DIR)/libveriload_basic.so: libveriload_basic.c | $(BUILD_DIR)
	$(CC) -nostdlib -fPIC -shared -Wl,-soname,libveriload_basic.so -o $@ $<

$(BUILD_DIR)/libveriload_dep_b.so: libveriload_dep_b.c | $(BUILD_DIR)
	$(CC) -nostdlib -fPIC -shared -Wl,-soname,libveriload_dep_b.so -o $@ $<

$(BUILD_DIR)/libveriload_dep_a.so: libveriload_dep_a.c $(BUILD_DIR)/libveriload_dep_b.so | $(BUILD_DIR)
	$(CC) -nostdlib -fPIC -shared -Wl,-soname,libveriload_dep_a.so -Wl,--no-as-needed -L$(BUILD_DIR) -lveriload_dep_b -Wl,--as-needed -o $@ $<

$(BUILD_DIR)/libveriload_missing_leaf.so: libveriload_missing_leaf.c | $(BUILD_DIR)
	$(CC) -nostdlib -fPIC -shared -Wl,-soname,libveriload_missing_leaf.so -o $@ $<

$(BUILD_DIR)/libveriload_missing_top.so: libveriload_missing_top.c $(BUILD_DIR)/libveriload_missing_leaf.so | $(BUILD_DIR)
	$(CC) -nostdlib -fPIC -shared -Wl,-soname,libveriload_missing_top.so -Wl,--no-as-needed -L$(BUILD_DIR) -lveriload_missing_leaf -Wl,--as-needed -o $@ $<

$(BUILD_DIR)/pos_no_deps: pos_no_deps.c | $(BUILD_DIR)
	$(CC) -nostdlib -fPIE -pie -Wl,-e,_start -Wl,--no-dynamic-linker -o $@ $<

$(BUILD_DIR)/pos_one_dep: pos_one_dep.c $(BUILD_DIR)/libveriload_basic.so | $(BUILD_DIR)
	$(CC) -nostdlib -fPIE -pie -Wl,-e,_start -Wl,--no-dynamic-linker -Wl,--no-as-needed -L$(BUILD_DIR) -lveriload_basic -Wl,--as-needed -o $@ $<

$(BUILD_DIR)/pos_transitive_dep: pos_transitive_dep.c $(BUILD_DIR)/libveriload_dep_a.so | $(BUILD_DIR)
	$(CC) -nostdlib -fPIE -pie -Wl,-e,_start -Wl,--no-dynamic-linker -Wl,--no-as-needed -L$(BUILD_DIR) -lveriload_dep_a -Wl,--as-needed -Wl,-rpath-link,$(BUILD_DIR) -o $@ $<

$(BUILD_DIR)/neg_missing_dep: neg_missing_dep.c $(BUILD_DIR)/libveriload_missing_top.so | $(BUILD_DIR)
	$(CC) -nostdlib -fPIE -pie -Wl,-e,_start -Wl,--no-dynamic-linker -Wl,--no-as-needed -L$(BUILD_DIR) -lveriload_missing_top -Wl,--as-needed -Wl,-rpath-link,$(BUILD_DIR) -o $@ $<

.PHONY: clean
clean:
	rm -rf $(BUILD_DIR)
