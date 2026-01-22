.PHONY: help build build-release test fmt clippy clean install check-version

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-20s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

build: ## Build debug binary
	cargo build

build-release: ## Build release binary
	cargo build --release

test: ## Run all tests
	cargo test --all-features

test-unit: ## Run unit tests only
	cargo test --lib

test-integration: ## Run integration tests only
	cargo test --test '*'

test-doc: ## Run documentation tests
	cargo test --doc

test-coverage: ## Generate test coverage report
	cargo tarpaulin --out Html --out Xml --all-features

test-watch: ## Run tests in watch mode (requires cargo-watch)
	cargo watch -x test

fmt: ## Format code
	cargo fmt

clippy: ## Run clippy linter
	cargo clippy --all-targets --all-features -- -D warnings

clean: ## Clean build artifacts
	cargo clean
	rm -rf npm/node_modules
	rm -f npm/*.tgz npm/*.tar.gz npm/*.zip
	rm -f npm/codex-mcp-rs npm/codex-mcp-rs.exe

install: build-release ## Install to system (requires sudo on Unix)
	@echo "Installing codex-mcp-rs..."
	@cp target/release/codex-mcp-rs /usr/local/bin/ || echo "Failed. Try: sudo make install"

check-version: ## Check version consistency across files
	@bash scripts/check-version.sh

check: fmt clippy test ## Run all checks (fmt, clippy, test)

ci: check build-release ## Run all CI checks

npm-pack: build-release ## Pack npm package for testing
	cd npm/codex-mcp-rs && npm pack

npm-install: npm-pack ## Install npm package locally for testing
	npm install -g npm/codex-mcp-rs/missdeer-codex-mcp-rs-*.tgz
