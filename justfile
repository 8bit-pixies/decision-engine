lint:
	@isort . --check-only 
	@ruff check . --fix
	@black --check .
	@echo "lint finished"

format:
	@isort .
	@black .
	@cargo fmt

test:
	@cargo test
	@pytest decision_engine_py/python/tests