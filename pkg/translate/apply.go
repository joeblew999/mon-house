package translate

import (
	"bufio"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

// LoadTask loads a translation task from a JSON file
// Single entry point for loading task files
func LoadTask(rootDir string, taskFile string) (*Task, error) {
	taskPath := filepath.Join(rootDir, taskFile)

	data, err := os.ReadFile(taskPath)
	if err != nil {
		return nil, fmt.Errorf("failed to read task file: %w", err)
	}

	var task Task
	if err := json.Unmarshal(data, &task); err != nil {
		return nil, fmt.Errorf("failed to parse task JSON: %w", err)
	}

	return &task, nil
}

// ValidateTask validates that a task has translations filled in
// Returns statistics about the task
func ValidateTask(task *Task) *ApplyStats {
	stats := &ApplyStats{}

	for _, file := range task.Files {
		stats.TotalExtractions += len(file.Extractions)
		for _, ext := range file.Extractions {
			if ext.TargetText != "" {
				stats.FilledExtractions++
			}
		}
	}

	return stats
}

// ApplyTranslations applies translations from a task to target files
// Single entry point for applying translations
func ApplyTranslations(rootDir string, task *Task) (*ApplyStats, error) {
	stats := ValidateTask(task)

	if stats.FilledExtractions == 0 {
		return stats, fmt.Errorf("no translations found in task file (all target_text fields are empty)")
	}

	// Apply translations to each file
	for _, file := range task.Files {
		// Count filled extractions for this file
		filledInFile := 0
		for _, ext := range file.Extractions {
			if ext.TargetText != "" {
				filledInFile++
			}
		}

		if filledInFile == 0 {
			stats.FilesSkipped++
			continue
		}

		targetPath := filepath.Join(rootDir, file.Target)

		// Apply based on file type
		var applyErr error
		if file.Type == "svg" {
			applyErr = applySVGTranslations(targetPath, file.Extractions)
		} else if file.Type == "md" || file.Type == "markdown" {
			applyErr = applyMarkdownTranslations(targetPath, file.Extractions)
		} else {
			stats.FilesSkipped++
			continue
		}

		if applyErr != nil {
			stats.FilesSkipped++
		} else {
			stats.FilesProcessed++
		}
	}

	return stats, nil
}

// DeleteTask deletes a task file
// Single entry point for task cleanup
func DeleteTask(rootDir string, taskFile string) error {
	taskPath := filepath.Join(rootDir, taskFile)
	if err := os.Remove(taskPath); err != nil {
		return fmt.Errorf("failed to delete task file: %w", err)
	}
	return nil
}

// applySVGTranslations applies translations to an SVG file
func applySVGTranslations(filePath string, extractions []TextExtraction) error {
	// Read the entire SVG file
	data, err := os.ReadFile(filePath)
	if err != nil {
		return err
	}

	content := string(data)

	// Apply each translation by replacing source text with target text
	for _, ext := range extractions {
		if ext.TargetText == "" {
			continue // Skip empty translations
		}

		// Replace the source text with target text
		// This is a simple string replacement approach
		content = strings.Replace(content, ">"+ext.SourceText+"<", ">"+ext.TargetText+"<", 1)
	}

	// Write back to file
	return os.WriteFile(filePath, []byte(content), 0644)
}

// applyMarkdownTranslations applies translations to a Markdown file
func applyMarkdownTranslations(filePath string, extractions []TextExtraction) error {
	// Read the file line by line
	file, err := os.Open(filePath)
	if err != nil {
		return err
	}
	defer file.Close()

	var lines []string
	scanner := bufio.NewScanner(file)
	for scanner.Scan() {
		lines = append(lines, scanner.Text())
	}

	if err := scanner.Err(); err != nil {
		return err
	}

	// Apply translations by line number
	for _, ext := range extractions {
		if ext.TargetText == "" {
			continue // Skip empty translations
		}

		// Line numbers are 1-indexed
		if ext.Line > 0 && ext.Line <= len(lines) {
			// Replace the entire line with the translated text
			lines[ext.Line-1] = ext.TargetText
		}
	}

	// Write back to file
	output := strings.Join(lines, "\n") + "\n"
	return os.WriteFile(filePath, []byte(output), 0644)
}
