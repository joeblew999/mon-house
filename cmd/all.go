package cmd

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/joeblew999/mon-house/internal/config"
	"github.com/joeblew999/mon-house/internal/generator"
	"github.com/joeblew999/mon-house/internal/injector"
	"github.com/joeblew999/mon-house/internal/semantic"
	"github.com/joeblew999/mon-house/internal/validator"
)

// HandleAll runs the complete workflow: generate CSS -> inject CSS -> validate SVGs -> semantic checks
// This is the SINGLE SOURCE OF TRUTH for the complete workflow
func HandleAll(args []string) {
	fmt.Println("=== mon-tool ALL - Complete Workflow ===")
	fmt.Println()
	fmt.Println("This workflow ensures:")
	fmt.Println("  ✓ Visual styles are up to date (CSS)")
	fmt.Println("  ✓ Drawings have required metadata (semantic)")
	fmt.Println("  ✓ Syntax is valid (SVG validation)")
	fmt.Println()

	// STEP 1: Generate CSS from drawing-standards.json
	fmt.Println("Step 1: Generating CSS from drawing-standards.json")

	// Look for drawing-standards.json in current dir or ../code/
	standardsPath := "drawing-standards.json"
	if _, err := os.Stat(standardsPath); os.IsNotExist(err) {
		standardsPath = "../code/drawing-standards.json"
	}

	cssOutputPath := "drawing-standards_gen.css"

	// Load drawing-standards.json
	input, err := config.LoadJSON(standardsPath)
	if err != nil {
		fmt.Fprintf(os.Stderr, "✗ Error reading %s: %v\n", standardsPath, err)
		os.Exit(1)
	}

	// Generate CSS
	css, err := generator.GenerateCSS(input)
	if err != nil {
		fmt.Fprintf(os.Stderr, "✗ Error generating CSS: %v\n", err)
		os.Exit(1)
	}

	// Write CSS to file
	if err := os.WriteFile(cssOutputPath, []byte(css), 0644); err != nil {
		fmt.Fprintf(os.Stderr, "✗ Error writing CSS file: %v\n", err)
		os.Exit(1)
	}
	fmt.Printf("✓ CSS generated: %s\n", cssOutputPath)
	fmt.Println()

	// STEP 2: Inject CSS into SVG files from drawings.json
	fmt.Println("Step 2: Injecting CSS into SVG files from drawings.json")
	drawingsPath := "drawings.json"

	// Read drawings config
	cfg, err := config.LoadDrawingsConfig(drawingsPath)
	if err != nil {
		fmt.Fprintf(os.Stderr, "✗ Error reading %s: %v\n", drawingsPath, err)
		os.Exit(1)
	}

	// Get base directory
	baseDir := filepath.Dir(drawingsPath)

	// Inject CSS into each SVG file
	for _, file := range cfg.Drawings.Files {
		svgPath := filepath.Join(baseDir, cfg.Drawings.BasePath, file.Path)
		fmt.Printf("  Injecting: %s\n", svgPath)

		if err := injector.InjectCSS(svgPath, css); err != nil {
			fmt.Fprintf(os.Stderr, "  ✗ Error: %v\n", err)
			continue
		}
		fmt.Printf("  ✓ Success\n")
	}
	fmt.Println()

	// STEP 3: Validate SVG files
	fmt.Println("Step 3: Validating SVG files")

	var svgPaths []string
	for _, file := range cfg.Drawings.Files {
		svgPath := filepath.Join(baseDir, cfg.Drawings.BasePath, file.Path)
		svgPaths = append(svgPaths, svgPath)
	}

	totalErrors := validator.ValidateFiles(svgPaths)
	fmt.Println()

	// STEP 4: Semantic validation
	fmt.Println("Step 4: Semantic validation (metadata completeness)")

	semanticErrors := 0
	for _, file := range cfg.Drawings.Files {
		svgPath := filepath.Join(baseDir, cfg.Drawings.BasePath, file.Path)

		errors, err := semantic.ValidateMetadata(svgPath, input)
		if err != nil {
			fmt.Printf("  ✗ Error validating %s: %v\n", file.Path, err)
			continue
		}

		if len(errors) == 0 {
			fmt.Printf("  ✓ %s - All required metadata present\n", file.Path)
		} else {
			fmt.Printf("  ⚠ %s - %d metadata issues\n", file.Path, len(errors))
			semanticErrors += len(errors)
		}
	}
	fmt.Println()

	// Summary
	fmt.Println("=== Summary ===")
	fmt.Printf("✓ CSS generated: %s\n", cssOutputPath)
	fmt.Printf("✓ CSS injected into %d SVG files\n", len(cfg.Drawings.Files))

	if totalErrors > 0 {
		fmt.Printf("⚠ %d syntax validation warnings\n", totalErrors)
	} else {
		fmt.Printf("✓ Syntax validation passed\n")
	}

	if semanticErrors > 0 {
		fmt.Printf("⚠ %d semantic metadata issues found\n", semanticErrors)
		fmt.Println()
		fmt.Println("Run 'mon-tool semantic validate <file>' for details")
	} else {
		fmt.Printf("✓ All semantic metadata present\n")
	}

	fmt.Println()
	if totalErrors > 0 || semanticErrors > 0 {
		fmt.Println("Complete - SVG files updated (warnings above)")
	} else {
		fmt.Println("Complete - All checks passed!")
	}
}
