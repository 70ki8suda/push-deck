.PHONY: dev lint test build bundle check

NPM ?= npm
CARGO ?= cargo
RUST_MANIFEST ?= src-tauri/Cargo.toml
TAURI ?= $(NPM) run tauri --
APP_NAME ?= Push Deck.app
APP_BUNDLE_PATH ?= src-tauri/target/release/bundle/macos/$(APP_NAME)
APP_INSTALL_DIR ?= $(HOME)/Applications
APP_INSTALL_PATH ?= $(APP_INSTALL_DIR)/$(APP_NAME)

dev:
	$(NPM) run dev:app

lint:
	$(NPM) run lint
	$(CARGO) check --manifest-path $(RUST_MANIFEST)

test:
	$(NPM) test
	$(CARGO) test --manifest-path $(RUST_MANIFEST)

build:
	$(NPM) run build
	$(CARGO) build --manifest-path $(RUST_MANIFEST)

bundle:
	mkdir -p "$(APP_INSTALL_DIR)"
	$(TAURI) build --bundles app --no-sign
	ditto "$(APP_BUNDLE_PATH)" "$(APP_INSTALL_PATH)"
	touch "$(APP_INSTALL_PATH)"

check: lint test
