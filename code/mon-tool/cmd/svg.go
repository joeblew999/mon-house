package cmd

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/joeblew999/mon-house/code/mon-tool/internal/config"
	"github.com/joeblew999/mon-house/code/mon-tool/internal/validator"
)

// HandleSVG handles SVG subcommands
func HandleSVG(args []string) {
	if len(args) < 1 {
		printSVGUsage()
		os.Exit(1)
	}

	subcommand := args[0]

	switch subcommand {
	case "validate":
		handleSVGValidate(args[1:])
	case "gen":
		handleSVGGen(args[1:])
	default:
		fmt.Fprintf(os.Stderr, "Unknown svg subcommand: %s\n\n", subcommand)
		printSVGUsage()
		os.Exit(1)
	}
}

func printSVGUsage() {
	fmt.Println("SVG commands:")
	fmt.Println("  svg validate [file...]         Validate SVG files (no args = use drawings.json)")
	fmt.Println("  svg gen element <type> [opts]  Generate element snippet")
	fmt.Println("  svg gen titleblock             Generate title block snippet")
	fmt.Println()
	fmt.Println("Validation checks:")
	fmt.Println("  - No inline styles (use CSS classes)")
	fmt.Println("  - No external stylesheets (use embedded <style>)")
	fmt.Println("  - Has embedded CSS")
	fmt.Println("  - All classes are defined in CSS")
	fmt.Println()
	fmt.Println("Examples:")
	fmt.Println("  mon-tool svg validate                       # Validate all files in drawings.json")
	fmt.Println("  mon-tool svg validate ../drawings/en/*/*.svg # Validate specific files")
	fmt.Println("  mon-tool svg gen element door --id=door-1 --x=100 --y=200")
}

func handleSVGValidate(args []string) {
	var svgPaths []string

	if len(args) == 0 {
		// No args - use drawings.json
		cfg, err := config.LoadDrawingsConfig("drawings.json")
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error reading drawings.json: %v\n", err)
			os.Exit(1)
		}

		// Build list of SVG paths from drawings.json
		for _, file := range cfg.Drawings.Files {
			svgPath := filepath.Join(cfg.Drawings.BasePath, file.Path)
			svgPaths = append(svgPaths, svgPath)
		}
	} else {
		// Use specified files
		svgPaths = args
	}

	totalErrors := validator.ValidateFiles(svgPaths)

	if totalErrors > 0 {
		os.Exit(1)
	}
}

func handleSVGGen(args []string) {
	if len(args) < 1 {
		fmt.Println("svg gen subcommands:")
		fmt.Println("  element <type> [opts]  Generate element snippet")
		fmt.Println("  titleblock             Generate title block snippet")
		os.Exit(1)
	}

	genType := args[0]

	switch genType {
	case "element":
		fmt.Fprintf(os.Stderr, "Element generation not yet implemented\n")
		fmt.Fprintf(os.Stderr, "Use generate-element tool for now\n")
		os.Exit(1)
	case "titleblock":
		fmt.Fprintf(os.Stderr, "Titleblock generation not yet implemented\n")
		fmt.Fprintf(os.Stderr, "Use generate-titleblock tool for now\n")
		os.Exit(1)
	default:
		fmt.Fprintf(os.Stderr, "Unknown gen type: %s\n", genType)
		os.Exit(1)
	}
}
