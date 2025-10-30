.PHONY: deps
deps:
	@echo "Installing dependencies..."
	go install github.com/cli/cli/v2/cmd/gh@latest
	@echo "Dependencies installed successfully"
