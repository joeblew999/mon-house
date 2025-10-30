package main

import (
	"fmt"
	"os"

	"github.com/joeblew999/mon-house/code/mon-tool/cmd"
)

func main() {
	if len(os.Args) < 2 {
		printUsage()
		os.Exit(1)
	}

	command := os.Args[1]

	switch command {
	case "all":
		cmd.HandleAll(os.Args[2:])
	case "css":
		cmd.HandleCSS(os.Args[2:])
	case "svg":
		cmd.HandleSVG(os.Args[2:])
	case "semantic":
		cmd.HandleSemantic(os.Args[2:])
	case "drawing":
		cmd.HandleDrawing(os.Args[2:])
	case "help", "-h", "--help":
		printUsage()
	default:
		fmt.Fprintf(os.Stderr, "Unknown command: %s\n\n", command)
		printUsage()
		os.Exit(1)
	}
}

func printUsage() {
	fmt.Println("mon-tool - Unified tool for mon-house SVG drawings")
	fmt.Println()
	fmt.Println("Usage: mon-tool <command> [options]")
	fmt.Println()
	fmt.Println("=== MAIN WORKFLOW ===")
	fmt.Println("  all                       Run complete workflow (generate → inject → validate)")
	fmt.Println()
	fmt.Println("=== INDIVIDUAL COMMANDS ===")
	fmt.Println()
	fmt.Println("CSS Commands:")
	fmt.Println("  css generate              Generate CSS from drawing-standards.json")
	fmt.Println("                            → Outputs to stdout or drawing-standards_gen.css")
	fmt.Println("  css inject <css-file>     Inject CSS into SVG files")
	fmt.Println("                            → Uses drawings.json to find SVG files")
	fmt.Println()
	fmt.Println("SVG Commands:")
	fmt.Println("  svg validate [files...]   Validate SVG files")
	fmt.Println("                            → No args: uses drawings.json")
	fmt.Println("                            → With args: validates specified files")
	fmt.Println("  svg gen element           Generate element snippet (not yet implemented)")
	fmt.Println("  svg gen titleblock        Generate title block snippet (not yet implemented)")
	fmt.Println()
	fmt.Println("Semantic Commands:")
	fmt.Println("  semantic validate <files> Validate semantic rules and metadata")
	fmt.Println("                            → Checks required metadata (data-width, etc.)")
	fmt.Println("                            → Validates metadata format (numeric, no units)")
	fmt.Println()
	fmt.Println("Drawing Commands:")
	fmt.Println("  drawing list              List all drawings from drawings.json")
	fmt.Println("  drawing info <path>       Show detailed info about a drawing")
	fmt.Println()
	fmt.Println("=== DATA FLOW ===")
	fmt.Println()
	fmt.Println("Unidirectional (Source → Derived):")
	fmt.Println("  1. drawing-standards.json  →  CSS generation  →  drawing-standards_gen.css")
	fmt.Println("  2. drawing-standards_gen.css + drawings.json  →  CSS injection  →  SVG files")
	fmt.Println("  3. SVG files  →  Validation (syntactic + semantic)  →  Pass/Fail report")
	fmt.Println("  4. SVG files  →  SPEC.md generation  →  Technical specs (TODO)")
	fmt.Println("  5. EN/ drawings  →  Translation  →  TH/ drawings (manual)")
	fmt.Println()
	fmt.Println("Bidirectional (Must Stay Synchronized):")
	fmt.Println("  6. plan.svg  ↔  section.svg  (linked views, x-coords must match) (TODO)")
	fmt.Println()
	fmt.Println("Examples:")
	fmt.Println("  mon-tool all                                # Run complete workflow")
	fmt.Println("  mon-tool css generate > drawing-standards_gen.css")
	fmt.Println("  mon-tool css inject drawing-standards_gen.css")
	fmt.Println("  mon-tool svg validate")
	fmt.Println("  mon-tool drawing list")
}
