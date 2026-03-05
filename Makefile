INSTALL_DIR := $(HOME)/.cargo/bin

.PHONY: build install clean

build:
	cargo build --release

install: build
	cp target/release/kagienv $(INSTALL_DIR)/kagienv
	@echo "Installed kagienv to $(INSTALL_DIR)/kagienv"

clean:
	cargo clean
