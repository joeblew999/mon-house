package commands

import (
	"fmt"
	"time"

	"github.com/joeblew999/mon-house/pkg/translate"
	"github.com/joeblew999/mon-house/pkg/translate/events"
)

// ApplyHandler handles ApplyCommand execution
// This is the COMMAND HANDLER (CQRS pattern) with Event Sourcing
type ApplyHandler struct {
	eventStore *events.Store
}

// NewApplyHandler creates a new ApplyHandler with event store
func NewApplyHandler(eventStore *events.Store) *ApplyHandler {
	return &ApplyHandler{
		eventStore: eventStore,
	}
}

// Handle executes an ApplyCommand
// This is a COMMAND HANDLER - it changes state (filesystem)
func (h *ApplyHandler) Handle(cmd *ApplyCommand) (*ApplyResult, error) {
	// Step 1: Validate command
	if err := cmd.Validate(); err != nil {
		return nil, fmt.Errorf("invalid apply command: %w", err)
	}

	// Step 2: Load task file (QUERY - no side effects)
	task, err := translate.LoadTask(cmd.RootDir, cmd.TaskFile)
	if err != nil {
		return nil, fmt.Errorf("failed to load task: %w", err)
	}

	// Emit TaskLoaded event
	if h.eventStore != nil {
		h.eventStore.Append(&events.TaskLoaded{
			BaseEvent: events.BaseEvent{
				Type:      "TaskLoaded",
				Occurred:  time.Now(),
				SessionID: h.eventStore.SessionID(),
			},
			TaskFile:        cmd.TaskFile,
			TargetLanguage:  task.TargetLanguage,
			FileCount:       len(task.Files),
			ExtractionCount: countExtractions(task),
			FilledCount:     countFilledExtractions(task),
		})
	}

	// Step 3: Validate task has translations (QUERY - no side effects)
	stats := translate.ValidateTask(task)
	if stats.FilledExtractions == 0 {
		return nil, fmt.Errorf("no translations found in task file (all target_text fields are empty)")
	}

	result := &ApplyResult{
		TotalExtractions:  stats.TotalExtractions,
		FilledExtractions: stats.FilledExtractions,
	}

	// Step 4: Apply translations if not dry-run (COMMAND - changes state)
	if !cmd.DryRun {
		applyStats, err := translate.ApplyTranslations(cmd.RootDir, task)
		if err != nil {
			return nil, fmt.Errorf("failed to apply translations: %w", err)
		}

		result.FilesProcessed = applyStats.FilesProcessed
		result.FilesSkipped = applyStats.FilesSkipped

		// Emit TranslationApplied events for each file
		if h.eventStore != nil {
			for _, file := range task.Files {
				appliedCount := 0
				skippedCount := 0
				for _, ext := range file.Extractions {
					if ext.TargetText != "" {
						appliedCount++
					} else {
						skippedCount++
					}
				}

				h.eventStore.Append(&events.TranslationApplied{
					BaseEvent: events.BaseEvent{
						Type:      "TranslationApplied",
						Occurred:  time.Now(),
						SessionID: h.eventStore.SessionID(),
					},
					FilePath:     file.Target,
					FileType:     file.Type,
					AppliedCount: appliedCount,
					SkippedCount: skippedCount,
				})
			}
		}

		// Step 5: Delete task file if all successful (COMMAND - changes state)
		if applyStats.FilesProcessed > 0 && applyStats.FilesSkipped == 0 && applyStats.FilledExtractions == applyStats.TotalExtractions {
			if err := translate.DeleteTask(cmd.RootDir, cmd.TaskFile); err != nil {
				// Don't fail the whole operation if we can't delete the task file
				// Just mark it as not deleted
				result.TaskFileDeleted = false
			} else {
				result.TaskFileDeleted = true

				// Emit TaskDeleted event
				if h.eventStore != nil {
					h.eventStore.Append(&events.TaskDeleted{
						BaseEvent: events.BaseEvent{
							Type:      "TaskDeleted",
							Occurred:  time.Now(),
							SessionID: h.eventStore.SessionID(),
						},
						TaskFile: cmd.TaskFile,
						Reason:   "completed",
					})
				}
			}
		}
	}

	return result, nil
}

// countExtractions counts total extractions in a task
func countExtractions(task *translate.Task) int {
	count := 0
	for _, file := range task.Files {
		count += len(file.Extractions)
	}
	return count
}

// countFilledExtractions counts filled extractions in a task
func countFilledExtractions(task *translate.Task) int {
	count := 0
	for _, file := range task.Files {
		for _, ext := range file.Extractions {
			if ext.TargetText != "" {
				count++
			}
		}
	}
	return count
}
