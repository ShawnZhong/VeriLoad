CC ?= cc
BUILD_DIR ?= build

LIBS := \
	$(BUILD_DIR)/libvlfoo.so \
	$(BUILD_DIR)/libvlb.so \
	$(BUILD_DIR)/libvla.so \
	$(BUILD_DIR)/libvlmissing.so

BINS := \
	$(BUILD_DIR)/pos_minimal_pie \
	$(BUILD_DIR)/pos_one_dso \
	$(BUILD_DIR)/pos_multi_dso_bfs \
	$(BUILD_DIR)/neg_missing_needed

.PHONY: all
all: $(LIBS) $(BINS)

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

$(BUILD_DIR)/libvlfoo.so: libvlfoo.S | $(BUILD_DIR)
	$(CC) -nostdlib -fPIC -shared -Wl,-soname,libvlfoo.so -o $@ $<

$(BUILD_DIR)/libvlb.so: libvlb.S | $(BUILD_DIR)
	$(CC) -nostdlib -fPIC -shared -Wl,-soname,libvlb.so -o $@ $<

$(BUILD_DIR)/libvla.so: libvla.S $(BUILD_DIR)/libvlb.so | $(BUILD_DIR)
	$(CC) -nostdlib -fPIC -shared -Wl,-soname,libvla.so -o $@ $< -L$(BUILD_DIR) -lvlb

$(BUILD_DIR)/libvlmissing.so: libvlfoo.S | $(BUILD_DIR)
	$(CC) -nostdlib -fPIC -shared -Wl,-soname,libvlmissing.so -o $@ $<

$(BUILD_DIR)/pos_minimal_pie: pos_minimal_pie.S | $(BUILD_DIR)
	$(CC) -nostdlib -fPIE -pie -Wl,-e,_start -o $@ $<

$(BUILD_DIR)/pos_one_dso: pos_one_dso.S $(BUILD_DIR)/libvlfoo.so | $(BUILD_DIR)
	$(CC) -nostdlib -fPIE -pie -Wl,-e,_start -o $@ $< -L$(BUILD_DIR) -lvlfoo

$(BUILD_DIR)/pos_multi_dso_bfs: pos_multi_dso_bfs.S $(BUILD_DIR)/libvla.so | $(BUILD_DIR)
	$(CC) -nostdlib -fPIE -pie -Wl,-e,_start -Wl,-rpath-link,$(BUILD_DIR) -o $@ $< -L$(BUILD_DIR) -lvla

$(BUILD_DIR)/neg_missing_needed: neg_missing_needed.S $(BUILD_DIR)/libvlmissing.so | $(BUILD_DIR)
	$(CC) -nostdlib -fPIE -pie -Wl,-e,_start -o $@ $< -L$(BUILD_DIR) -lvlmissing

.PHONY: clean
clean:
	rm -rf $(BUILD_DIR)
