SHELL := /bin/bash

export PATH := $(HOME)/.cargo/bin:$(PATH)
export NVM_DIR := $(HOME)/.nvm

define setup_nvm
	source $(NVM_DIR)/nvm.sh
endef

.DEFAULT_GOAL := help

.PHONY: install dev check lint format build run clean help

install: ## Install npm dependencies
	@$(setup_nvm) && npm install

dev: ## Start the Tauri development server
	@$(setup_nvm) && npm run tauri dev

check: ## Run Rust and TypeScript type checking without building
	@cargo check --manifest-path src-tauri/Cargo.toml
	@$(setup_nvm) && npx tsc --noEmit

lint: ## Run ESLint on the frontend
	@$(setup_nvm) && npx eslint src

format: ## Format the codebase with Prettier
	@$(setup_nvm) && npx prettier --write .

build: ## Build the application for release
	@$(setup_nvm) && npm run tauri build

run: ## Run the last built binary
	@if [ "$(OS)" = "Windows_NT" ]; then \
		src-tauri/target/release/mpgrab.exe; \
	else \
		src-tauri/target/release/mpgrab; \
	fi

clean: ## Remove all build artifacts
	@cargo clean --manifest-path src-tauri/Cargo.toml
	@rm -rf dist

help: ## List all available targets
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-12s\033[0m %s\n", $$1, $$2}'
