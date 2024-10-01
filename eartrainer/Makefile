# Environment variables for the library paths
ANDROID_AARCH_LINUX_ANDROID_LIBC ?= ""
ANDROID_ARM_LINUX_ANDROIDEABI_LIBC ?= ""

# Destination directories for the librariesx#libc++_shared.so
LIBC_NAME= libc++_shared.so
DEST_ARM64_LIB_DIR = ./build/rust_lib_eartrainer/jniLibs/debug/arm64-v8a
DEST_ARMEABI_LIB_DIR = ./build/rust_lib_eartrainer/jniLibs/debug/armeabi-v7a

# Default device name (can be overridden by `make run DEVICE=<device_name>`)
DEVICE ?= ""

# Flutter run command
FLUTTER_RUN = flutter run


create_dirs:
	@mkdir -p $(DEST_ARM64_LIB_DIR)
	@mkdir -p $(DEST_ARMEABI_LIB_DIR)

# Target to copy the libraries
copy_libraries: create_dirs
	@if [ -z "$(ANDROID_AARCH_LINUX_ANDROID_LIBC)" ]; then \
		echo "ANDROID_AARCH_LINUX_ANDROID_LIBC is not set"; \
		exit 1; \
	else \
		echo "Copying $(ANDROID_AARCH_LINUX_ANDROID_LIBC) to $(DEST_ARM64_LIB_DIR)"; \
		rm $(DEST_ARM64_LIB_DIR)/$(LIBC_NAME) -f; \
		cp $(ANDROID_AARCH_LINUX_ANDROID_LIBC) $(DEST_ARM64_LIB_DIR); \
	fi
	@if [ -z "$(ANDROID_ARM_LINUX_ANDROIDEABI_LIBC)" ]; then \
		echo "ANDROID_ARM_LINUX_ANDROIDEABI_LIBC is not set"; \
		exit 1; \
	else \
		echo "Copying $(ANDROID_ARM_LINUX_ANDROIDEABI_LIBC) to $(DEST_ARMEABI_LIB_DIR)"; \
		rm $(DEST_ARMEABI_LIB_DIR)/$(LIBC_NAME) -f; \
		cp $(ANDROID_ARM_LINUX_ANDROIDEABI_LIBC) $(DEST_ARMEABI_LIB_DIR); \
	fi
	@echo "Libraries copied successfully."

# Target to run the Flutter app
run: copy_libraries
	@echo "Running Flutter app on device: $(DEVICE)"
	@if [ "$(DEVICE)" = "" ]; then \
		$(FLUTTER_RUN); \
	else \
		$(FLUTTER_RUN) -d $(DEVICE); \
	fi

.PHONY: create_dirs copy_libraries run
