package commands

import (
	"fmt"
	"os"
	"path/filepath"
	"time"

	"github.com/joeblew999/mon-house/pkg/translate"
	"github.com/joeblew999/mon-house/pkg/translate/events"
)

// SyncHandler handles SyncCommand execution
// This is the COMMAND HANDLER (CQRS pattern) with Event Sourcing
type SyncHandler struct {
	eventStore *events.Store
}

// NewSyncHandler creates a new SyncHandler with event store
func NewSyncHandler(eventStore *events.Store) *SyncHandler {
	return &SyncHandler{
		eventStore: eventStore,
	}
}

// Handle executes a SyncCommand
// This is a COMMAND HANDLER - it changes state (filesystem)
func (h *SyncHandler) Handle(cmd *SyncCommand) (*SyncResult, error) {
	// Step 1: Validate command
	if err := cmd.Validate(); err != nil {
		return nil, fmt.Errorf("invalid sync command: %w", err)
	}

	result := &SyncResult{
		TasksGenerated: []string{},
	}

	// Step 2: Load configuration
	config, err := translate.LoadConfig(cmd.RootDir)
	if err != nil {
		return nil, fmt.Errorf("failed to load config: %w", err)
	}

	// Step 3: Find source directory
	sourceDir := filepath.Join(cmd.RootDir, config.Source.Folder)

	// Step 4: Find target config
	var targetConfig *translate.TargetConfig
	for _, target := range config.Targets {
		if target.Language == cmd.TargetLang {
			targetConfig = &target
			break
		}
	}
	if targetConfig == nil {
		return nil, fmt.Errorf("target language %s not found in config", cmd.TargetLang)
	}

	// Step 5: Scan source and plan sync actions (QUERY - no side effects)
	actions, err := translate.ScanSource(cmd.RootDir, sourceDir, *targetConfig)
	if err != nil {
		return nil, fmt.Errorf("failed to scan source: %w", err)
	}

	// Step 6: Get statistics (QUERY - no side effects)
	mkdirs, copies, deletes := translate.GetSyncStats(actions)
	result.DirectoriesCreated = mkdirs
	result.FilesCopied = copies
	result.FilesDeleted = deletes

	// Step 7: Execute sync if not dry-run (COMMAND - changes state)
	if !cmd.DryRun {
		if err := translate.ExecuteSync(actions); err != nil {
			return nil, fmt.Errorf("failed to execute sync: %w", err)
		}

		// Emit events for each action performed
		if h.eventStore != nil {
			for _, action := range actions {
				switch action.Action {
				case "mkdir":
					h.eventStore.Append(&events.DirectoryCreated{
						BaseEvent: events.BaseEvent{
							Type:      "DirectoryCreated",
							Occurred:  time.Now(),
							SessionID: h.eventStore.SessionID(),
						},
						Path: action.Target,
					})
				case "copy":
					size := int64(0)
					if info, err := os.Stat(action.Source); err == nil {
						size = info.Size()
					}
					h.eventStore.Append(&events.FileCopied{
						BaseEvent: events.BaseEvent{
							Type:      "FileCopied",
							Occurred:  time.Now(),
							SessionID: h.eventStore.SessionID(),
						},
						SourcePath: action.Source,
						TargetPath: action.Target,
						Size:       size,
						FileType:   action.Type,
					})
				case "delete":
					h.eventStore.Append(&events.FileDeleted{
						BaseEvent: events.BaseEvent{
							Type:      "FileDeleted",
							Occurred:  time.Now(),
							SessionID: h.eventStore.SessionID(),
						},
						Path:   action.Target,
						Reason: "not_in_source",
					})
				}
			}
		}
	}

	// Step 8: Get translatable files (QUERY - no side effects)
	filesToTranslate := translate.GetTranslatableFiles(actions)

	// Step 9: Generate task file if not dry-run (COMMAND - changes state)
	if !cmd.DryRun && len(filesToTranslate) > 0 {
		extractionCount, err := translate.GenerateTask(cmd.RootDir, *targetConfig, filesToTranslate, config.Paths.Tasks)
		if err != nil {
			return nil, fmt.Errorf("failed to generate task: %w", err)
		}

		taskFile := fmt.Sprintf("%s/translate-%s.json", config.Paths.Tasks, cmd.TargetLang)
		result.TasksGenerated = append(result.TasksGenerated, taskFile)

		// Emit TaskGenerated event
		if h.eventStore != nil {
			h.eventStore.Append(&events.TaskGenerated{
				BaseEvent: events.BaseEvent{
					Type:      "TaskGenerated",
					Occurred:  time.Now(),
					SessionID: h.eventStore.SessionID(),
				},
				TaskFile:        taskFile,
				TargetLanguage:  cmd.TargetLang,
				FileCount:       len(filesToTranslate),
				ExtractionCount: extractionCount,
			})
		}
	}

	return result, nil
}
