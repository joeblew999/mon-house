package cmd

import (
	"fmt"
	"os"

	"github.com/joeblew999/mon-house/code/mon-tool/internal/config"
	"github.com/joeblew999/mon-house/code/mon-tool/internal/semantic"
)

// HandleSemantic handles semantic subcommands
func HandleSemantic(args []string) {
	if len(args) < 1 {
		printSemanticUsage()
		os.Exit(1)
	}

	subcommand := args[0]

	switch subcommand {
	case "validate":
		handleSemanticValidate(args[1:])
	default:
		fmt.Fprintf(os.Stderr, "Unknown semantic subcommand: %s\n\n", subcommand)
		printSemanticUsage()
		os.Exit(1)
	}
}

func printSemanticUsage() {
	fmt.Println("Semantic commands:")
	fmt.Println("  semantic validate <file> [file...]  Validate semantic rules and metadata")
	fmt.Println()
	fmt.Println("Semantic validation checks:")
	fmt.Println("  - Required metadata present (data-width, data-height, etc.)")
	fmt.Println("  - Metadata format correct (numeric values, no units)")
	fmt.Println("  - Elements have required properties")
	fmt.Println()
	fmt.Println("Examples:")
	fmt.Println("  mon-tool semantic validate plan.svg")
	fmt.Println("  mon-tool semantic validate plan.svg section.svg")
}

func handleSemanticValidate(args []string) {
	if len(args) < 1 {
		fmt.Fprintf(os.Stderr, "Usage: mon-tool semantic validate <svg-file> [svg-file...]\n")
		os.Exit(1)
	}

	// Load drawing-standards.json
	standardsPath := "drawing-standards.json"
	if _, err := os.Stat(standardsPath); os.IsNotExist(err) {
		standardsPath = "../code/drawing-standards.json"
	}

	standardsData, err := config.LoadJSON(standardsPath)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error loading drawing-standards.json: %v\n", err)
		os.Exit(1)
	}

	totalErrors := 0

	// Validate each SVG file
	for _, svgPath := range args {
		fmt.Printf("Validating: %s\n", svgPath)

		errors, err := semantic.ValidateMetadata(svgPath, standardsData)
		if err != nil {
			fmt.Fprintf(os.Stderr, "  Error: %v\n", err)
			continue
		}

		if len(errors) == 0 {
			fmt.Println("  ✓ All required metadata present")
		} else {
			fmt.Println(semantic.FormatMetadataErrors(errors))
			totalErrors += len(errors)
		}
	}

	if totalErrors > 0 {
		fmt.Printf("\n%d total semantic errors\n", totalErrors)
		os.Exit(1)
	}
}
