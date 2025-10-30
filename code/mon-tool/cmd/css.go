package cmd

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/joeblew999/mon-house/code/mon-tool/internal/config"
	"github.com/joeblew999/mon-house/code/mon-tool/internal/generator"
	"github.com/joeblew999/mon-house/code/mon-tool/internal/injector"
)

// HandleCSS handles CSS subcommands
func HandleCSS(args []string) {
	if len(args) < 1 {
		printCSSUsage()
		os.Exit(1)
	}

	subcommand := args[0]

	switch subcommand {
	case "generate":
		handleCSSGenerate(args[1:])
	case "inject":
		handleCSSInject(args[1:])
	default:
		fmt.Fprintf(os.Stderr, "Unknown css subcommand: %s\n\n", subcommand)
		printCSSUsage()
		os.Exit(1)
	}
}

func printCSSUsage() {
	fmt.Println("CSS commands:")
	fmt.Println("  css generate [standards.json]    Generate CSS from drawing-standards.json")
	fmt.Println("  css inject [css-file]             Inject CSS into SVG files (uses drawings.json)")
	fmt.Println()
	fmt.Println("Examples:")
	fmt.Println("  mon-tool css generate > drawing-standards_gen.css")
	fmt.Println("  mon-tool css generate drawing-standards.json > output.css")
	fmt.Println("  mon-tool css inject drawing-standards_gen.css")
}

func handleCSSGenerate(args []string) {
	// Get the JSON file path
	jsonPath := filepath.Join("code", "drawing-standards.json")
	if len(args) > 0 {
		jsonPath = args[0]
	}

	// Load JSON
	input, err := config.LoadJSON(jsonPath)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error reading JSON file: %v\n", err)
		os.Exit(1)
	}

	// Generate CSS
	css, err := generator.GenerateCSS(input)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error generating CSS: %v\n", err)
		os.Exit(1)
	}

	// Output CSS
	fmt.Print(css)
}

func handleCSSInject(args []string) {
	if len(args) < 1 {
		fmt.Fprintf(os.Stderr, "Usage: mon-tool css inject <css-file>\n")
		os.Exit(1)
	}

	cssPath := args[0]
	drawingsPath := "drawings.json"

	// Read CSS file
	cssContent, err := os.ReadFile(cssPath)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error reading CSS file: %v\n", err)
		os.Exit(1)
	}

	// Read drawings config
	cfg, err := config.LoadDrawingsConfig(drawingsPath)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error reading drawings config: %v\n", err)
		os.Exit(1)
	}

	// Get base directory (where drawings.json is located)
	baseDir := filepath.Dir(drawingsPath)

	// Process each SVG file
	for _, file := range cfg.Drawings.Files {
		svgPath := filepath.Join(baseDir, cfg.Drawings.BasePath, file.Path)

		fmt.Printf("Injecting CSS into: %s\n", svgPath)

		if err := injector.InjectCSS(svgPath, string(cssContent)); err != nil {
			fmt.Fprintf(os.Stderr, "  ✗ Error: %v\n", err)
			continue
		}

		fmt.Printf("  ✓ Success\n")
	}
}
