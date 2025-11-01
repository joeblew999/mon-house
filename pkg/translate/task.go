package translate

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

// GenerateTask generates a translation task JSON file
// Single entry point for task generation
// Returns the total number of extractions in the task
func GenerateTask(rootDir string, target TargetConfig, files []string, tasksPath string) (int, error) {
	// Build file list with source and target paths and extractions
	var taskFiles []TaskFile
	totalExtractions := 0
	for _, targetPath := range files {
		relTargetPath, _ := filepath.Rel(rootDir, targetPath)

		// Derive source path
		sourcePath := deriveSourcePath(relTargetPath, target)

		// Determine file type
		fileType := "other"
		if strings.HasSuffix(targetPath, ".svg") {
			fileType = "svg"
		} else if strings.HasSuffix(targetPath, ".md") {
			fileType = "md"
		}

		// Extract text from source file
		sourceFullPath := filepath.Join(rootDir, sourcePath)
		extractions, err := ExtractText(sourceFullPath, fileType)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Warning: failed to extract text from %s: %v\n", sourcePath, err)
		}

		totalExtractions += len(extractions)

		taskFiles = append(taskFiles, TaskFile{
			Source:      sourcePath,
			Target:      relTargetPath,
			Type:        fileType,
			Extractions: extractions,
		})
	}

	// Build task structure
	task := Task{
		Task:             fmt.Sprintf("Translate English to %s for architectural drawings", target.LanguageName),
		SourceLanguage:   "en",
		TargetLanguage:   target.Language,
		LanguageName:     target.LanguageName,
		Files:            taskFiles,
		TranslationNotes: target.TranslationNotes,
		Instructions: map[string][]string{
			"svg": {
				"Translate all <text> element content",
				"Translate <title> element content",
				"DO NOT translate: attributes, CSS classes, coordinates, numbers",
				"Preserve: XML structure, formatting, indentation",
			},
			"markdown": {
				"Translate all text content",
				"DO NOT translate: code blocks, file paths, URLs",
				"Preserve: markdown formatting, structure, links",
			},
		},
	}

	// Create tasks directory (path from config, not hardcoded)
	tasksDir := filepath.Join(rootDir, tasksPath)
	if err := os.MkdirAll(tasksDir, 0755); err != nil {
		return 0, fmt.Errorf("failed to create tasks directory: %w", err)
	}

	// Write task JSON file
	taskPath := filepath.Join(tasksDir, fmt.Sprintf("translate-%s.json", target.Language))
	jsonData, err := json.MarshalIndent(task, "", "  ")
	if err != nil {
		return 0, fmt.Errorf("failed to marshal JSON: %w", err)
	}

	if err := os.WriteFile(taskPath, jsonData, 0644); err != nil {
		return 0, fmt.Errorf("failed to write task file: %w", err)
	}

	return totalExtractions, nil
}

// deriveSourcePath derives the source path from a target path
func deriveSourcePath(relTargetPath string, target TargetConfig) string {
	sourcePath := relTargetPath
	// Convert from target folder to source folder (e.g., drawings/th -> drawings/en)
	sourcePath = strings.Replace(sourcePath, target.Folder+"/", "drawings/en/", 1)
	// Reverse rename rules (e.g., .th.md -> .md)
	for oldExt, newExt := range target.RenameRules {
		if strings.HasSuffix(sourcePath, newExt) {
			sourcePath = strings.TrimSuffix(sourcePath, newExt) + oldExt
			break
		}
	}
	return sourcePath
}
