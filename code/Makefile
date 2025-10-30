# Paths
CODE_DIR = $(shell pwd)
TEST_DIR = $(CODE_DIR)/../test
MON_TOOL = $(CODE_DIR)/mon-tool/mon-tool
MON_TOOL_SRC = $(CODE_DIR)/mon-tool

.PHONY: help build test all clean

help:
	@echo "Makefile for mon-house"
	@echo ""
	@echo "make test    Test on test/test.svg (SAFE - use this first!)"
	@echo "make all     Run on production drawings (updates real files)"
	@echo "make build   Build mon-tool binary"
	@echo "make clean   Remove generated CSS files"

# Build mon-tool if it doesn't exist or source changed
$(MON_TOOL): $(shell find $(MON_TOOL_SRC) -name '*.go')
	@echo "Building mon-tool..."
	@cd $(MON_TOOL_SRC) && go build
	@echo "✓ mon-tool built"

build: $(MON_TOOL)

test: $(MON_TOOL)
	@echo "=== TESTING on test/test.svg (safe) ==="
	@echo ""
	@cd $(TEST_DIR) && $(MON_TOOL) all
	@echo ""
	@echo "✓ Test complete - check test/test.svg"
	@echo "If this looks good, run 'make all' to update production drawings"

all: $(MON_TOOL)
	@echo "=== PRODUCTION - Updating real drawings ==="
	@echo ""
	@cd $(CODE_DIR) && $(MON_TOOL) all

clean:
	@rm -f $(CODE_DIR)/drawing-standards_gen.css
	@rm -f $(TEST_DIR)/drawing-standards_gen.css
	@echo "✓ Cleaned generated CSS files"
