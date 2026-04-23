.PHONY: build develop release test check clean distclean lint fmt help

VENV := .venv
BIN := $(VENV)/bin

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

$(VENV):
	@command -v uv >/dev/null 2>&1 || { echo "Installing uv..."; pip install uv; }
	uv venv $(VENV)

develop: $(VENV) ## Build and install in current Python env (dev mode)
	uv pip install -r requirements-dev.txt
	$(BIN)/maturin develop

build: ## Build release wheel
	$(BIN)/maturin build --release

release: build ## Build release wheel (alias for build)

test: develop ## Run tests
	$(BIN)/pytest test_dustr.py -v

check: ## Run cargo check
	cargo check

lint: ## Run clippy and Python linters
	cargo clippy -- -D warnings
	$(BIN)/ruff check python/

fmt: ## Format Rust and Python code
	cargo fmt
	$(BIN)/ruff format python/

clean: ## Remove build artifacts
	cargo clean
	rm -rf target/wheels/

distclean: clean ## Remove build artifacts and Python caches
	rm -rf __pycache__ python/dustr/__pycache__
	rm -rf *.egg-info python/*.egg-info
	rm -rf .ruff_cache .mypy_cache
	find . -name '*.pyc' -delete
	find . -name '__pycache__' -type d -delete
