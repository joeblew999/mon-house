# mon-house Translation Makefile
# Keep It Simple: Build tool, sync translations (test first, then production)

# Paths
MON_TOOL := mon-tool

.PHONY: help build test prod clean

help:
	@echo "mon-house Translation Workflow"
	@echo ""
	@echo "  make build    Build mon-tool"
	@echo "  make test     Test translation sync (safe - uses examples/test/)"
	@echo "  make prod     Production translation sync (uses examples/production/)"
	@echo "  make clean    Remove binary"

# Build mon-tool
build:
	@go build -o $(MON_TOOL)
	@echo "✓ Built $(MON_TOOL)"

# Test translation sync (safe)
test: build
	@echo "Testing translation sync..."
	@cd examples/test && ../../$(MON_TOOL) translate sync
	@echo "✓ Test complete"

# Production translation sync
prod: build
	@echo "Production translation sync..."
	@cd examples/production && ../../$(MON_TOOL) translate sync
	@echo "✓ Production sync complete"

# Clean
clean:
	@rm -f $(MON_TOOL)
	@echo "✓ Cleaned"
