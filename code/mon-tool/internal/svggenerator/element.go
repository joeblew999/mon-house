package svggenerator

import (
	"fmt"
	"strings"

	"github.com/itchyny/gojq"
)

// ElementRequest represents a request to generate an SVG element
type ElementRequest struct {
	ElementType string            // e.g., "window", "door", "wall-exterior"
	ID          string            // Optional ID for the element
	Position    Position          // Where to place the element
	Dimensions  Dimensions        // Element dimensions
	Metadata    map[string]string // Additional data-* attributes
}

// Position represents element placement
type Position struct {
	X      float64 // X coordinate in pixels
	Y      float64 // Y coordinate in pixels
	Rotate float64 // Rotation in degrees (optional)
}

// Dimensions represents element size
type Dimensions struct {
	Width  float64 // Width in meters
	Height float64 // Height in meters
	Length float64 // Length in meters (for walls, beams)
}

// GenerateElement creates SVG markup for a single element
func GenerateElement(req ElementRequest, standardsData interface{}) (string, error) {
	// Get element definition from drawing-standards.json
	elemDef, err := getElementDefinition(req.ElementType, standardsData)
	if err != nil {
		return "", fmt.Errorf("element type '%s' not found in drawing-standards.json: %w", req.ElementType, err)
	}

	// Check if grouping is required
	groupingRequired := isGroupingRequired(req.ElementType, standardsData)

	// Get required metadata attributes
	requiredAttrs := getRequiredAttributes(req.ElementType, standardsData)

	// Validate that all required metadata is provided
	if err := validateRequiredMetadata(req, requiredAttrs); err != nil {
		return "", fmt.Errorf("missing required metadata: %w", err)
	}

	// Generate SVG based on element type
	svg := ""
	if groupingRequired {
		svg = generateGroupedElement(req, elemDef, requiredAttrs)
	} else {
		svg = generateSimpleElement(req, elemDef)
	}

	return svg, nil
}

// getElementDefinition extracts element definition from standards
func getElementDefinition(elementType string, standardsData interface{}) (map[string]interface{}, error) {
	query := fmt.Sprintf(".drawingStandards.elements[\"%s\"]", elementType)
	q, err := gojq.Parse(query)
	if err != nil {
		return nil, err
	}

	iter := q.Run(standardsData)
	v, ok := iter.Next()
	if !ok || v == nil {
		return nil, fmt.Errorf("element not found")
	}

	elemDef, ok := v.(map[string]interface{})
	if !ok {
		return nil, fmt.Errorf("invalid element definition format")
	}

	return elemDef, nil
}

// isGroupingRequired checks if element requires <g> wrapper
func isGroupingRequired(elementType string, standardsData interface{}) bool {
	query := fmt.Sprintf(".drawingStandards.elements[\"%s\"].requiredMetadata.grouping", elementType)
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

	return grouping == "required" || grouping == "recommended"
}

// getRequiredAttributes extracts required metadata attributes
func getRequiredAttributes(elementType string, standardsData interface{}) map[string]bool {
	result := make(map[string]bool)

	query := fmt.Sprintf(".drawingStandards.elements[\"%s\"].requiredMetadata.attributes", elementType)
	q, err := gojq.Parse(query)
	if err != nil {
		return result
	}

	iter := q.Run(standardsData)
	v, ok := iter.Next()
	if !ok || v == nil {
		return result
	}

	attrs, ok := v.(map[string]interface{})
	if !ok {
		return result
	}

	for attrName, attrData := range attrs {
		attrMap, ok := attrData.(map[string]interface{})
		if !ok {
			continue
		}

		if required, ok := attrMap["required"].(bool); ok && required {
			result[attrName] = true
		}
	}

	return result
}

// validateRequiredMetadata checks if all required attributes are present
func validateRequiredMetadata(req ElementRequest, requiredAttrs map[string]bool) error {
	var missing []string

	for attr := range requiredAttrs {
		// Check if attribute is provided in request
		found := false

		// Check dimensions
		if attr == "data-width" && req.Dimensions.Width > 0 {
			found = true
		}
		if attr == "data-height" && req.Dimensions.Height > 0 {
			found = true
		}
		if attr == "data-length" && req.Dimensions.Length > 0 {
			found = true
		}

		// Check metadata map
		if _, ok := req.Metadata[attr]; ok {
			found = true
		}

		if !found {
			missing = append(missing, attr)
		}
	}

	if len(missing) > 0 {
		return fmt.Errorf("missing required attributes: %s", strings.Join(missing, ", "))
	}

	return nil
}

// generateGroupedElement creates a <g> group with child elements
func generateGroupedElement(req ElementRequest, elemDef map[string]interface{}, requiredAttrs map[string]bool) string {
	var sb strings.Builder

	// Generate <g> opening tag with class and metadata
	sb.WriteString(fmt.Sprintf("  <g"))

	// Add ID if provided
	if req.ID != "" {
		sb.WriteString(fmt.Sprintf(" id=\"%s\"", req.ID))
	}

	// Add class
	className := req.ElementType
	if cssClass, ok := elemDef["cssClass"].(string); ok {
		className = cssClass
	}
	sb.WriteString(fmt.Sprintf(" class=\"%s\"", className))

	// Add required metadata attributes
	if req.Dimensions.Width > 0 {
		sb.WriteString(fmt.Sprintf(" data-width=\"%.1f\"", req.Dimensions.Width))
	}
	if req.Dimensions.Height > 0 {
		sb.WriteString(fmt.Sprintf(" data-height=\"%.1f\"", req.Dimensions.Height))
	}
	if req.Dimensions.Length > 0 {
		sb.WriteString(fmt.Sprintf(" data-length=\"%.1f\"", req.Dimensions.Length))
	}

	// Add additional metadata
	for key, value := range req.Metadata {
		sb.WriteString(fmt.Sprintf(" %s=\"%s\"", key, value))
	}

	sb.WriteString(">\n")

	// Add title element (for accessibility and tooltips)
	description := ""
	if desc, ok := elemDef["description"].(string); ok {
		description = desc
	}
	sb.WriteString(fmt.Sprintf("    <title>%s</title>\n", description))

	// Generate child element(s) based on element type
	childSVG := generateChildElements(req, elemDef)
	sb.WriteString(childSVG)

	// Close </g>
	sb.WriteString("  </g>\n")

	return sb.String()
}

// generateSimpleElement creates a simple SVG element without grouping
func generateSimpleElement(req ElementRequest, elemDef map[string]interface{}) string {
	// For simple elements like lines, rectangles without required metadata
	return generateChildElements(req, elemDef)
}

// generateChildElements creates the actual SVG shapes
func generateChildElements(req ElementRequest, elemDef map[string]interface{}) string {
	// Determine SVG shape based on element type
	elementType := req.ElementType

	switch {
	case strings.Contains(elementType, "wall"):
		return generateWall(req)
	case elementType == "window":
		return generateWindow(req)
	case elementType == "door", elementType == "door-sliding":
		return generateDoor(req, elementType)
	case elementType == "beam":
		return generateBeam(req)
	case elementType == "furniture", strings.Contains(elementType, "kitchen"):
		return generateRectangle(req)
	default:
		// Default: generate a rectangle
		return generateRectangle(req)
	}
}

// generateWall creates a wall line
func generateWall(req ElementRequest) string {
	x1 := req.Position.X
	y1 := req.Position.Y
	x2 := x1 + req.Dimensions.Length*100 // Convert meters to pixels
	y2 := y1

	// Handle rotation if specified
	if req.Position.Rotate == 90 || req.Position.Rotate == 270 {
		x2 = x1
		y2 = y1 + req.Dimensions.Length*100
	}

	return fmt.Sprintf("    <line class=\"%s\" x1=\"%.0f\" y1=\"%.0f\" x2=\"%.0f\" y2=\"%.0f\"/>\n",
		req.ElementType, x1, y1, x2, y2)
}

// generateWindow creates a window rectangle
func generateWindow(req ElementRequest) string {
	x := req.Position.X
	y := req.Position.Y
	width := 8.0 // Fixed visual width for plan view (represents wall thickness)
	height := req.Dimensions.Width * 100 // Window width becomes height in plan view

	return fmt.Sprintf("    <rect class=\"window\" x=\"%.0f\" y=\"%.0f\" width=\"%.0f\" height=\"%.0f\"/>\n",
		x, y, width, height)
}

// generateDoor creates a door with arc
func generateDoor(req ElementRequest, doorType string) string {
	x := req.Position.X
	y := req.Position.Y
	width := req.Dimensions.Width * 100 // Convert meters to pixels

	var sb strings.Builder

	if doorType == "door-sliding" {
		// Sliding door - just a rectangle
		sb.WriteString(fmt.Sprintf("    <rect class=\"door-sliding\" x=\"%.0f\" y=\"%.0f\" width=\"%.0f\" height=\"8\"/>\n",
			x, y, width))
	} else {
		// Swinging door - line + arc
		sb.WriteString(fmt.Sprintf("    <line class=\"door\" x1=\"%.0f\" y1=\"%.0f\" x2=\"%.0f\" y2=\"%.0f\"/>\n",
			x, y, x+width, y))
		sb.WriteString(fmt.Sprintf("    <path class=\"door-arc\" d=\"M %.0f %.0f Q %.0f %.0f %.0f %.0f\"/>\n",
			x, y, x+width*0.2, y-width*0.2, x+width*0.5, y-width*0.5))
	}

	return sb.String()
}

// generateBeam creates a beam rectangle
func generateBeam(req ElementRequest) string {
	x := req.Position.X
	y := req.Position.Y
	width := req.Dimensions.Length * 100 // Beam length
	height := 20.0                       // Fixed beam height in pixels

	return fmt.Sprintf("    <rect class=\"beam\" x=\"%.0f\" y=\"%.0f\" width=\"%.0f\" height=\"%.0f\"/>\n",
		x, y, width, height)
}

// generateRectangle creates a generic rectangle
func generateRectangle(req ElementRequest) string {
	x := req.Position.X
	y := req.Position.Y
	width := req.Dimensions.Width * 100
	height := req.Dimensions.Height * 100

	return fmt.Sprintf("    <rect class=\"%s\" x=\"%.0f\" y=\"%.0f\" width=\"%.0f\" height=\"%.0f\"/>\n",
		req.ElementType, x, y, width, height)
}
