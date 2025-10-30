.PHONY: deps
deps:
	@echo "Installing dependencies..."
	@echo "Note: Requires network access to github.com and golang.org repositories"
	GOPROXY=direct go install github.com/cli/cli/v2/cmd/gh@latest
	@echo "Dependencies installed successfully"
	@echo "Verifying gh CLI installation..."
	@command -v gh >/dev/null 2>&1 && gh version || echo "Warning: gh not found in PATH. Check GOPATH/bin is in your PATH"
