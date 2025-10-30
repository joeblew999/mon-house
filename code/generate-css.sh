#!/bin/bash
# Generate CSS from drawing-standards.json using Go tool

cd "$(dirname "$0")/generate-css" && go run main.go ../drawing-standards.json
