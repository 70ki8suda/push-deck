.PHONY: dev lint test build check

NPM ?= npm
CARGO ?= cargo
RUST_MANIFEST ?= src-tauri/Cargo.toml

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

check: lint test
