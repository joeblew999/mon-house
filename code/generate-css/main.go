package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/itchyny/gojq"
)

func main() {
	// Get the JSON file path
	jsonPath := filepath.Join("code", "drawing-standards.json")
	if len(os.Args) > 1 {
		jsonPath = os.Args[1]
	}

	// Read the JSON file
	data, err := os.ReadFile(jsonPath)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error reading JSON file: %v\n", err)
		os.Exit(1)
	}

	// Parse JSON
	var input interface{}
	if err := json.Unmarshal(data, &input); err != nil {
		fmt.Fprintf(os.Stderr, "Error parsing JSON: %v\n", err)
		os.Exit(1)
	}

	// Generate CSS
	css, err := generateCSS(input)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error generating CSS: %v\n", err)
		os.Exit(1)
	}

	// Output CSS
	fmt.Println(css)
}

func generateCSS(input interface{}) (string, error) {
	var sb strings.Builder

	sb.WriteString("/* Generated from drawing-standards.json */\n\n")
	sb.WriteString("/* Element visual styles */\n")

	// Query for all elements with visual properties
	query, err := gojq.Parse(".drawingStandards.elements | to_entries[] | {name: .key, visual: .value.visual}")
	if err != nil {
		return "", fmt.Errorf("error parsing query: %w", err)
	}

	iter := query.Run(input)
	for {
		v, ok := iter.Next()
		if !ok {
			break
		}
		if err, ok := v.(error); ok {
			return "", fmt.Errorf("query error: %w", err)
		}

		element := v.(map[string]interface{})
		name := element["name"].(string)
		visual, ok := element["visual"].(map[string]interface{})
		if !ok || visual == nil {
			continue
		}

		// Resolve $ref references in visual properties
		visual = resolveRefs(visual, input)

		// Generate CSS rule
		css := formatCSSRule(name, visual)
		sb.WriteString(css)
	}

	sb.WriteString("\n/* Label styles */\n")

	// Query for label styles
	labelQuery, err := gojq.Parse(`.drawingStandards.elements | to_entries[] | select(.value.requiredMetadata.childElements.label != null) | {name: .key, label: .value.requiredMetadata.childElements.label}`)
	if err != nil {
		return "", fmt.Errorf("error parsing label query: %w", err)
	}

	iter = labelQuery.Run(input)
	for {
		v, ok := iter.Next()
		if !ok {
			break
		}
		if _, ok := v.(error); ok {
			// No labels found, skip
			continue
		}

		element := v.(map[string]interface{})
		name := element["name"].(string)
		label, ok := element["label"].(map[string]interface{})
		if !ok || label == nil {
			continue
		}

		// Resolve $ref references
		label = resolveRefs(label, input)

		// Generate label CSS rule
		labelName := name + "-label"
		css := formatCSSRule(labelName, label)
		sb.WriteString(css)
	}

	return sb.String(), nil
}

func formatCSSRule(className string, props map[string]interface{}) string {
	var sb strings.Builder

	sb.WriteString("      .")
	sb.WriteString(className)
	sb.WriteString(" { ")

	// CSS properties we want to output (filter out metadata)
	cssProps := []string{
		"stroke", "strokeWidth", "strokeDasharray", "fill", "opacity",
		"fontSize", "fontFamily", "fontWeight", "fontStyle",
	}

	// Convert properties to CSS
	first := true
	for _, key := range cssProps {
		value, exists := props[key]
		if !exists {
			continue
		}

		if !first {
			sb.WriteString("; ")
		}
		first = false

		// Convert camelCase to kebab-case
		cssKey := camelToKebab(key)

		// Format value
		cssValue := formatCSSValue(key, value)

		sb.WriteString(cssKey)
		sb.WriteString(": ")
		sb.WriteString(cssValue)
	}

	sb.WriteString("; }\n")
	return sb.String()
}

func camelToKebab(s string) string {
	var result strings.Builder
	for i, r := range s {
		if i > 0 && r >= 'A' && r <= 'Z' {
			result.WriteRune('-')
		}
		result.WriteRune(r)
	}
	return strings.ToLower(result.String())
}

func formatCSSValue(key string, value interface{}) string {
	switch key {
	case "strokeDasharray":
		// Format array as comma-separated values
		if arr, ok := value.([]interface{}); ok {
			parts := make([]string, len(arr))
			for i, v := range arr {
				parts[i] = fmt.Sprintf("%v", v)
			}
			return strings.Join(parts, ",")
		}
	case "fontSize":
		// Add px unit to font size
		return fmt.Sprintf("%vpx", value)
	case "strokeWidth":
		// Stroke width is unitless in SVG (implicitly px)
		return fmt.Sprintf("%v", value)
	case "fontFamily":
		// Extract name from font family object or use as string
		if obj, ok := value.(map[string]interface{}); ok {
			if name, exists := obj["name"]; exists {
				return fmt.Sprintf("%v", name)
			}
		}
	case "fontWeight":
		// Font weight should be unquoted
		return fmt.Sprintf("%v", value)
	}

	// Default formatting
	return fmt.Sprintf("%v", value)
}

func resolveRefs(props map[string]interface{}, input interface{}) map[string]interface{} {
	result := make(map[string]interface{})

	for key, value := range props {
		if strVal, ok := value.(string); ok && strings.HasPrefix(strVal, "$ref:") {
			// Resolve reference
			refPath := strings.TrimPrefix(strVal, "$ref:")
			resolved := resolveRef(refPath, input)
			if resolved != nil {
				result[key] = resolved
			} else {
				result[key] = value
			}
		} else {
			result[key] = value
		}
	}

	return result
}

func resolveRef(path string, input interface{}) interface{} {
	// Prepend drawingStandards to the path if not already there
	fullPath := "drawingStandards." + path

	// Split path and traverse JSON
	parts := strings.Split(fullPath, ".")
	current := input

	for _, part := range parts {
		if m, ok := current.(map[string]interface{}); ok {
			current = m[part]
		} else {
			return nil
		}
	}

	// If we got an object with "name" and "fallback" (fontFamily), return the name
	if m, ok := current.(map[string]interface{}); ok {
		if name, exists := m["name"]; exists {
			return name
		}
	}

	return current
}
