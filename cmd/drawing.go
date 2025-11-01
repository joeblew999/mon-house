package cmd

import (
	"fmt"
	"os"

	"github.com/joeblew999/mon-house/internal/config"
)

// HandleDrawing handles drawing subcommands
func HandleDrawing(args []string) {
	if len(args) < 1 {
		printDrawingUsage()
		os.Exit(1)
	}

	subcommand := args[0]

	switch subcommand {
	case "list":
		handleDrawingList(args[1:])
	case "info":
		handleDrawingInfo(args[1:])
	default:
		fmt.Fprintf(os.Stderr, "Unknown drawing subcommand: %s\n\n", subcommand)
		printDrawingUsage()
		os.Exit(1)
	}
}

func printDrawingUsage() {
	fmt.Println("Drawing commands:")
	fmt.Println("  drawing list           List all drawings from drawings.json")
	fmt.Println("  drawing info <path>    Show detailed info about a drawing")
	fmt.Println()
	fmt.Println("Examples:")
	fmt.Println("  mon-tool drawing list")
	fmt.Println("  mon-tool drawing info en/existing/plan.svg")
}

func handleDrawingList(args []string) {
	cfg, err := config.LoadDrawingsConfig("drawings.json")
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error reading drawings.json: %v\n", err)
		os.Exit(1)
	}

	fmt.Printf("Drawings (v%s):\n", cfg.Drawings.Version)
	fmt.Printf("Base path: %s\n", cfg.Drawings.BasePath)
	fmt.Printf("Scale: %d px per %s\n\n", cfg.Drawings.Scale.PixelsPerMeter, cfg.Drawings.Scale.Unit)

	for i, file := range cfg.Drawings.Files {
		fmt.Printf("%d. %s\n", i+1, file.Path)
		fmt.Printf("   Type: %s\n", file.Type)
		fmt.Printf("   Status: %s\n", file.Status)
		fmt.Printf("   Size: %dx%d\n", file.Width, file.Height)
		if file.Title != "" {
			fmt.Printf("   Title: %s\n", file.Title)
		}
		fmt.Println()
	}
}

func handleDrawingInfo(args []string) {
	if len(args) < 1 {
		fmt.Fprintf(os.Stderr, "Usage: mon-tool drawing info <path>\n")
		os.Exit(1)
	}

	path := args[0]

	cfg, err := config.LoadDrawingsConfig("drawings.json")
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error reading drawings.json: %v\n", err)
		os.Exit(1)
	}

	// Find the drawing
	var found *config.DrawingFile
	for _, file := range cfg.Drawings.Files {
		if file.Path == path {
			found = &file
			break
		}
	}

	if found == nil {
		fmt.Fprintf(os.Stderr, "Drawing not found: %s\n", path)
		os.Exit(1)
	}

	// Display info
	fmt.Printf("Drawing: %s\n\n", found.Path)
	fmt.Printf("Type:      %s\n", found.Type)
	fmt.Printf("Status:    %s\n", found.Status)
	fmt.Printf("Size:      %dx%d\n", found.Width, found.Height)
	fmt.Printf("ViewBox:   %s\n", found.ViewBox)
	if found.Title != "" {
		fmt.Printf("Title:     %s\n", found.Title)
	}
	if found.Subtitle != "" {
		fmt.Printf("Subtitle:  %s\n", found.Subtitle)
	}
	if found.ScaleText != "" {
		fmt.Printf("Scale:     %s\n", found.ScaleText)
	}
	fmt.Println()

	// Display paper size and scale info
	fmt.Printf("Paper:     %s %s (%dx%dmm)\n",
		cfg.Drawings.PaperSize.Format,
		cfg.Drawings.PaperSize.Orientation,
		cfg.Drawings.PaperSize.WidthMM,
		cfg.Drawings.PaperSize.HeightMM)
	fmt.Printf("Scale:     %d px per %s\n",
		cfg.Drawings.Scale.PixelsPerMeter,
		cfg.Drawings.Scale.Unit)
}
