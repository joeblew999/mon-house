package validator

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

// ValidateFile validates a single SVG file
func ValidateFile(svgPath string) ([]string, error) {
	content, err := os.ReadFile(svgPath)
	if err != nil {
		return nil, err
	}

	return ValidateSVG(string(content)), nil
}

// ValidateSVG validates SVG content against mon-house standards
func ValidateSVG(content string) []string {
	var errors []string

	// Check for inline stroke/fill attributes (should use CSS classes)
	inlineStyles := []string{}
	for _, attr := range []string{`stroke="`, `fill="`, `stroke-width="`, `font-size="`} {
		if strings.Contains(content, attr) {
			// Exclude allowed inline styles (like stroke on circles, fill on rect backgrounds)
			count := strings.Count(content, attr)
			if count > 2 { // Allow a few for backgrounds, etc.
				inlineStyles = append(inlineStyles, fmt.Sprintf("%s (%d times)", strings.TrimSuffix(attr, `="`), count))
			}
		}
	}
	if len(inlineStyles) > 0 {
		errors = append(errors, fmt.Sprintf("Inline style attributes found: %s", strings.Join(inlineStyles, ", ")))
	}

	// Check for external stylesheet
	if strings.Contains(content, "<?xml-stylesheet") {
		errors = append(errors, "Uses external stylesheet - run 'make all' to embed CSS")
	}

	// Check for embedded CSS
	if !strings.Contains(content, "<style>") {
		errors = append(errors, "Missing <style> section - run 'make all' to inject CSS")
	}

	// Check CSS classes are defined
	if strings.Contains(content, "<style>") {
		styleContent := extractStyleContent(content)
		classes := extractClasses(content)

		for _, className := range classes {
			if !strings.Contains(styleContent, "."+className+" {") && !strings.Contains(styleContent, "."+className+"{") {
				errors = append(errors, fmt.Sprintf("Class '%s' used but not defined in CSS", className))
			}
		}
	}

	return errors
}

// ValidateFiles validates multiple SVG files
func ValidateFiles(svgPaths []string) int {
	totalErrors := 0
	for _, svgPath := range svgPaths {
		errors, err := ValidateFile(svgPath)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error reading %s: %v\n", svgPath, err)
			continue
		}

		if len(errors) == 0 {
			fmt.Printf("✓ %s\n", filepath.Base(svgPath))
		} else {
			fmt.Printf("✗ %s (%d errors):\n", filepath.Base(svgPath), len(errors))
			for _, err := range errors {
				fmt.Printf("  - %s\n", err)
			}
			fmt.Println()
			totalErrors += len(errors)
		}
	}

	if totalErrors > 0 {
		fmt.Printf("\n%d total validation errors\n", totalErrors)
	}

	return totalErrors
}

func extractClasses(content string) []string {
	var classes []string
	seen := make(map[string]bool)

	lines := strings.Split(content, "\n")
	for _, line := range lines {
		if strings.Contains(line, `class="`) {
			start := strings.Index(line, `class="`) + 7
			rest := line[start:]
			end := strings.Index(rest, `"`)
			if end > 0 {
				className := rest[:end]
				if !seen[className] {
					classes = append(classes, className)
					seen[className] = true
				}
			}
		}
	}

	return classes
}

func extractStyleContent(content string) string {
	start := strings.Index(content, "<style>")
	end := strings.Index(content, "</style>")
	if start == -1 || end == -1 {
		return ""
	}
	return content[start:end]
}
