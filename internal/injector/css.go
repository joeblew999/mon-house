package injector

import (
	"fmt"
	"os"
	"regexp"
	"strings"
)

// InjectCSS injects CSS into an SVG file's <style> section
func InjectCSS(svgPath string, css string) error {
	// Read SVG file
	content, err := os.ReadFile(svgPath)
	if err != nil {
		return fmt.Errorf("reading SVG: %w", err)
	}

	svgContent := string(content)

	// Remove <?xml-stylesheet?> directive if present
	xmlStylesheetRegex := regexp.MustCompile(`<\?xml-stylesheet[^>]*\?>\s*`)
	svgContent = xmlStylesheetRegex.ReplaceAllString(svgContent, "")

	// Format CSS for embedding with proper indentation
	indentedCSS := indentCSS(css)

	// Check if SVG already has a <style> section
	styleRegex := regexp.MustCompile(`(?s)<defs>\s*<style>.*?</style>\s*</defs>`)
	defsRegex := regexp.MustCompile(`<defs>\s*</defs>`)

	newStyle := fmt.Sprintf("<defs>\n    <style>\n%s    </style>\n  </defs>", indentedCSS)

	if styleRegex.MatchString(svgContent) {
		// Replace existing <style> section
		svgContent = styleRegex.ReplaceAllString(svgContent, newStyle)
	} else if defsRegex.MatchString(svgContent) {
		// Replace empty <defs> with <style>
		svgContent = defsRegex.ReplaceAllString(svgContent, newStyle)
	} else {
		// Insert after <svg> tag
		svgTagRegex := regexp.MustCompile(`(<svg[^>]*>)`)
		svgContent = svgTagRegex.ReplaceAllString(svgContent, "$1\n  "+newStyle)
	}

	// Write updated SVG
	if err := os.WriteFile(svgPath, []byte(svgContent), 0644); err != nil {
		return fmt.Errorf("writing SVG: %w", err)
	}

	return nil
}

func indentCSS(css string) string {
	lines := strings.Split(css, "\n")
	var result strings.Builder

	for _, line := range lines {
		if strings.TrimSpace(line) != "" {
			result.WriteString("      ")
			result.WriteString(line)
			result.WriteString("\n")
		} else {
			result.WriteString("\n")
		}
	}

	return result.String()
}
