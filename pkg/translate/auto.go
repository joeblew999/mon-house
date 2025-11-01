package translate

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"

	"github.com/joeblew999/mon-house/pkg/translate/ai"
)

// AutoTranslate uses AI to automatically fill in translations (HEADLESS mode)
// This enables fully automated translation without human intervention
func AutoTranslate(rootDir string, taskFile string, translator ai.Translator) (*Task, *ai.TranslationResponse, error) {
	// Step 1: Load the task file
	task, err := LoadTask(rootDir, taskFile)
	if err != nil {
		return nil, nil, fmt.Errorf("failed to load task: %w", err)
	}

	// Step 2: Validate task structure
	if len(task.Files) == 0 {
		return nil, nil, fmt.Errorf("task has no files to translate")
	}

	// Step 3: Build translation request for AI
	req := &ai.TranslationRequest{
		SourceLanguage: task.SourceLanguage,
		TargetLanguage: task.TargetLanguage,
		LanguageName:   task.LanguageName,
		Domain:         "architectural drawings",
		Notes:          task.TranslationNotes,
		Items:          []ai.TranslationItem{},
	}

	// Extract terminology from notes if present
	// TODO: Parse this from translate.json or drawing-standards.json
	req.Terminology = map[string]string{
		"envelope":      "แนวเปลือกอาคาร",
		"wall-exterior": "ผนังภายนอก",
		"roof":          "หลังคา",
		"foundation":    "ฐานราก",
	}

	// Collect all items that need translation
	itemID := 0
	fileItemMap := make(map[string][]int) // maps item ID to [fileIdx, extIdx]

	for fileIdx, file := range task.Files {
		for extIdx, ext := range file.Extractions {
			if ext.TargetText == "" {
				itemIDStr := fmt.Sprintf("%d", itemID)
				req.Items = append(req.Items, ai.TranslationItem{
					ID:         itemIDStr,
					Context:    ext.Context,
					SourceText: ext.SourceText,
					TargetText: "",
				})
				fileItemMap[itemIDStr] = []int{fileIdx, extIdx}
				itemID++
			}
		}
	}

	if len(req.Items) == 0 {
		return task, nil, fmt.Errorf("all translations already filled (nothing to do)")
	}

	// Step 4: Call AI translator (HEADLESS - fully automated)
	resp, err := translator.Translate(req)
	if err != nil {
		return nil, nil, fmt.Errorf("AI translation failed: %w", err)
	}

	// Step 5: Apply translations back to task
	for _, item := range resp.Translations {
		if indices, ok := fileItemMap[item.ID]; ok {
			fileIdx := indices[0]
			extIdx := indices[1]
			task.Files[fileIdx].Extractions[extIdx].TargetText = item.TargetText
		}
	}

	return task, resp, nil
}

// SaveTask saves a task back to its JSON file
func SaveTask(rootDir string, taskFile string, task *Task) error {
	fullPath := filepath.Join(rootDir, taskFile)

	jsonData, err := json.MarshalIndent(task, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal task: %w", err)
	}

	if err := os.WriteFile(fullPath, jsonData, 0644); err != nil {
		return fmt.Errorf("failed to write task file: %w", err)
	}

	return nil
}

// TranslationProgress returns statistics about translation progress
type TranslationProgress struct {
	TotalFiles        int
	TotalExtractions  int
	FilledExtractions int
	EmptyExtractions  int
	PercentComplete   int
}

// GetTranslationProgress calculates translation progress for a task
func GetTranslationProgress(task *Task) TranslationProgress {
	progress := TranslationProgress{
		TotalFiles: len(task.Files),
	}

	for _, file := range task.Files {
		for _, ext := range file.Extractions {
			progress.TotalExtractions++
			if ext.TargetText != "" {
				progress.FilledExtractions++
			} else {
				progress.EmptyExtractions++
			}
		}
	}

	if progress.TotalExtractions > 0 {
		progress.PercentComplete = (progress.FilledExtractions * 100) / progress.TotalExtractions
	}

	return progress
}
