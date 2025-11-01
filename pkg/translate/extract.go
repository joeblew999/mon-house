package translate

import (
	"bufio"
	"encoding/xml"
	"fmt"
	"io"
	"os"
	"strings"
)

// ExtractText extracts translatable text from a file
// Single entry point for text extraction
func ExtractText(filePath string, fileType string) ([]TextExtraction, error) {
	switch fileType {
	case "svg":
		return extractSVGText(filePath)
	case "md":
		return extractMarkdownText(filePath)
	default:
		return nil, fmt.Errorf("unsupported file type: %s", fileType)
	}
}

// extractSVGText extracts translatable text from SVG file
func extractSVGText(filePath string) ([]TextExtraction, error) {
	file, err := os.Open(filePath)
	if err != nil {
		return nil, err
	}
	defer file.Close()

	var extractions []TextExtraction
	decoder := xml.NewDecoder(file)
	lineNum := 1
	var currentPath []string // Track element path for XPath

	for {
		token, err := decoder.Token()
		if err == io.EOF {
			break
		}
		if err != nil {
			return nil, err
		}

		switch elem := token.(type) {
		case xml.StartElement:
			currentPath = append(currentPath, elem.Name.Local)
			lineNum++

		case xml.EndElement:
			if len(currentPath) > 0 {
				currentPath = currentPath[:len(currentPath)-1]
			}

		case xml.CharData:
			text := strings.TrimSpace(string(elem))
			if text == "" {
				continue
			}

			// Only extract from text and title elements
			if len(currentPath) > 0 {
				lastElem := currentPath[len(currentPath)-1]
				if lastElem == "text" || lastElem == "title" {
					xpath := "/" + strings.Join(currentPath, "/")
					extractions = append(extractions, TextExtraction{
						Line:       lineNum,
						XPath:      xpath,
						SourceText: text,
						TargetText: "",
					})
				}
			}
		}
	}

	return extractions, nil
}

// extractMarkdownText extracts translatable text from Markdown file
func extractMarkdownText(filePath string) ([]TextExtraction, error) {
	file, err := os.Open(filePath)
	if err != nil {
		return nil, err
	}
	defer file.Close()

	var extractions []TextExtraction
	scanner := bufio.NewScanner(file)
	lineNum := 0
	inCodeBlock := false

	for scanner.Scan() {
		lineNum++
		line := scanner.Text()
		trimmed := strings.TrimSpace(line)

		// Skip empty lines
		if trimmed == "" {
			continue
		}

		// Track code blocks
		if strings.HasPrefix(trimmed, "```") {
			inCodeBlock = !inCodeBlock
			continue
		}

		// Skip code blocks
		if inCodeBlock {
			continue
		}

		// Determine context
		context := "paragraph"
		displayText := trimmed

		if strings.HasPrefix(trimmed, "#") {
			context = "heading"
		} else if strings.HasPrefix(trimmed, "- ") || strings.HasPrefix(trimmed, "* ") {
			context = "list-item"
		} else if strings.HasPrefix(trimmed, ">") {
			context = "blockquote"
		}

		// Extract text (keep markdown formatting like **bold**)
		extractions = append(extractions, TextExtraction{
			Line:       lineNum,
			Context:    context,
			SourceText: displayText,
			TargetText: "",
		})
	}

	if err := scanner.Err(); err != nil {
		return nil, err
	}

	return extractions, nil
}
