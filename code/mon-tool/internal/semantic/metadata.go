package semantic

import (
	
	"fmt"
	"os"
	"regexp"
	"strings"

	"github.com/itchyny/gojq"
)

// MetadataError represents a metadata validation error
type MetadataError struct {
	ElementID   string
	ClassName   string
	Attribute   string
	Issue       string
	LineApprox  int
}

// ValidateMetadata validates that SVG elements have required metadata from drawing-standards.json
func ValidateMetadata(svgPath string, standardsData interface{}) ([]MetadataError, error) {
	// Read SVG
	content, err := os.ReadFile(svgPath)
	if err != nil {
		return nil, err
	}

	var errors []MetadataError

	// Extract all elements with class attributes
	elements := extractElementsWithClass(string(content))

	// For each element, check required metadata
	for _, elem := range elements {
		// Get required metadata for this class from standards
		required := getRequiredMetadata(elem.Class, standardsData)

		// Check if this element type requires grouping
		groupingRequired := isGroupingRequired(elem.Class, standardsData)

		// If grouping is required, only validate elements that have an id attribute
		// (these are parent <g> elements, not child elements)
		if groupingRequired && !elem.HasID {
			continue
		}

		// Check each required attribute
		for attrName, attrSpec := range required {
			var value string
			var exists bool

			// Special case: 'id' is stored in elem.ID, not elem.Attributes
			if attrName == "id" {
				exists = elem.HasID
				value = elem.ID
			} else {
				value, exists = elem.Attributes[attrName]
			}

			if !exists && attrSpec.Required {
				errors = append(errors, MetadataError{
					ElementID:  elem.ID,
					ClassName:  elem.Class,
					Attribute:  attrName,
					Issue:      fmt.Sprintf("Missing required attribute '%s'", attrName),
					LineApprox: elem.Line,
				})
				continue
			}

			if exists {
				// Validate format
				if attrSpec.Type == "number" {
					if !isNumeric(value) {
						errors = append(errors, MetadataError{
							ElementID:  elem.ID,
							ClassName:  elem.Class,
							Attribute:  attrName,
							Issue:      fmt.Sprintf("Value '%s' is not numeric (unit must be omitted)", value),
							LineApprox: elem.Line,
						})
					}
				}
			}
		}
	}

	return errors, nil
}

// SVGElement represents a parsed SVG element
type SVGElement struct {
	ID         string
	Class      string
	Tag        string
	Attributes map[string]string
	Line       int
	HasID      bool // True if element has an id attribute
}

// AttributeSpec represents metadata attribute specification
type AttributeSpec struct {
	Required    bool
	Type        string
	Unit        string
	Example     string
	Description string
}

func extractElementsWithClass(svgContent string) []SVGElement {
	var elements []SVGElement

	// Simple regex-based extraction (good enough for validation)
	classRegex := regexp.MustCompile(`class="([^"]+)"`)
	idRegex := regexp.MustCompile(`id="([^"]+)"`)
	dataRegex := regexp.MustCompile(`data-([a-z-]+)="([^"]+)"`)

	lines := strings.Split(svgContent, "\n")
	for lineNum, line := range lines {
		if strings.Contains(line, `class="`) {
			classMatch := classRegex.FindStringSubmatch(line)
			if classMatch == nil {
				continue
			}

			elem := SVGElement{
				Class:      classMatch[1],
				Attributes: make(map[string]string),
				Line:       lineNum + 1,
			}

			// Extract ID if present
			if idMatch := idRegex.FindStringSubmatch(line); idMatch != nil {
				elem.ID = idMatch[1]
				elem.HasID = true
			} else {
				elem.ID = fmt.Sprintf("(unnamed %s)", elem.Class)
				elem.HasID = false
			}

			// Extract all data-* attributes from current line
			dataMatches := dataRegex.FindAllStringSubmatch(line, -1)
			for _, match := range dataMatches {
				attrName := "data-" + match[1]
				attrValue := match[2]
				elem.Attributes[attrName] = attrValue
			}

			// CRITICAL FIX: Check subsequent lines for data-* attributes (multi-line elements)
			// Continue reading until we hit '>' (end of opening tag)
			if !strings.Contains(line, ">") {
				for i := lineNum + 1; i < len(lines); i++ {
					continueLine := lines[i]

					// Extract data-* attributes from continuation line
					dataMatches := dataRegex.FindAllStringSubmatch(continueLine, -1)
					for _, match := range dataMatches {
						attrName := "data-" + match[1]
						attrValue := match[2]
						elem.Attributes[attrName] = attrValue
					}

					// Also check for id on continuation lines (rare but possible)
					if !elem.HasID {
						if idMatch := idRegex.FindStringSubmatch(continueLine); idMatch != nil {
							elem.ID = idMatch[1]
							elem.HasID = true
						}
					}

					// Stop if we reach the end of the opening tag
					if strings.Contains(continueLine, ">") {
						break
					}
				}
			}

			elements = append(elements, elem)
		}
	}

	return elements
}

func getRequiredMetadata(className string, standardsData interface{}) map[string]AttributeSpec {
	result := make(map[string]AttributeSpec)

	// Query JSON for this element's required metadata
	query := fmt.Sprintf(".drawingStandards.elements[\"%s\"].requiredMetadata.attributes", className)
	q, err := gojq.Parse(query)
	if err != nil {
		return result
	}

	iter := q.Run(standardsData)
	v, ok := iter.Next()
	if !ok || v == nil {
		return result
	}

	// Convert to map
	attrs, ok := v.(map[string]interface{})
	if !ok {
		return result
	}

	for attrName, attrData := range attrs {
		attrMap, ok := attrData.(map[string]interface{})
		if !ok {
			continue
		}

		spec := AttributeSpec{}
		if req, ok := attrMap["required"].(bool); ok {
			spec.Required = req
		}
		if t, ok := attrMap["type"].(string); ok {
			spec.Type = t
		}
		if u, ok := attrMap["unit"].(string); ok {
			spec.Unit = u
		}
		if ex, ok := attrMap["example"].(string); ok {
			spec.Example = ex
		}
		if desc, ok := attrMap["description"].(string); ok {
			spec.Description = desc
		}

		result[attrName] = spec
	}

	return result
}

func isNumeric(s string) bool {
	// Check if string is a valid number (int or float)
	matched, _ := regexp.MatchString(`^-?\d+(\.\d+)?$`, s)
	return matched
}

func isGroupingRequired(className string, standardsData interface{}) bool {
	// Query JSON for this element's grouping requirement
	query := fmt.Sprintf(".drawingStandards.elements[\"%s\"].requiredMetadata.grouping", className)
	q, err := gojq.Parse(query)
	if err != nil {
		return false
	}

	iter := q.Run(standardsData)
	v, ok := iter.Next()
	if !ok || v == nil {
		return false
	}

	grouping, ok := v.(string)
	if !ok {
		return false
	}

	// Treat both "required" and "recommended" as grouping required
	// This ensures child elements without id are skipped during validation
	return grouping == "required" || grouping == "recommended"
}

// FormatMetadataErrors formats metadata errors for display
func FormatMetadataErrors(errors []MetadataError) string {
	if len(errors) == 0 {
		return "✓ All required metadata present"
	}

	var sb strings.Builder
	sb.WriteString(fmt.Sprintf("✗ %d metadata errors:\n\n", len(errors)))

	for _, err := range errors {
		sb.WriteString(fmt.Sprintf("  Line ~%d: %s (class=%s)\n", err.LineApprox, err.ElementID, err.ClassName))
		sb.WriteString(fmt.Sprintf("    %s: %s\n", err.Attribute, err.Issue))
		sb.WriteString("\n")
	}

	return sb.String()
}
