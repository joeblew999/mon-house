package cmd

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/joeblew999/mon-house/pkg/translate"
	"github.com/joeblew999/mon-house/pkg/translate/ai"
	"github.com/joeblew999/mon-house/pkg/translate/commands"
	"github.com/joeblew999/mon-house/pkg/translate/events"
)

// HandleTranslate handles the translate command using CQRS pattern
func HandleTranslate(args []string) {
	if len(args) == 0 {
		printTranslateUsage()
		os.Exit(1)
	}

	subcommand := args[0]

	switch subcommand {
	case "sync":
		dryRun := false
		if len(args) > 1 && args[1] == "--dry-run" {
			dryRun = true
		}
		handleTranslateSync(dryRun)
	case "apply":
		if len(args) < 2 {
			fmt.Fprintf(os.Stderr, "Error: translate apply requires a task file path\n\n")
			printTranslateUsage()
			os.Exit(1)
		}
		dryRun := false
		if len(args) > 2 && args[2] == "--dry-run" {
			dryRun = true
		}
		handleTranslateApply(args[1], dryRun)
	case "auto":
		if len(args) < 2 {
			fmt.Fprintf(os.Stderr, "Error: translate auto requires a task file path\n\n")
			printTranslateUsage()
			os.Exit(1)
		}
		apiKey := os.Getenv("ANTHROPIC_API_KEY")
		if len(args) > 2 && strings.HasPrefix(args[2], "--api-key=") {
			apiKey = strings.TrimPrefix(args[2], "--api-key=")
		}
		handleTranslateAuto(args[1], apiKey)
	case "events":
		handleTranslateEvents()
	case "help", "-h", "--help":
		printTranslateUsage()
	default:
		fmt.Fprintf(os.Stderr, "Unknown translate subcommand: %s\n\n", subcommand)
		printTranslateUsage()
		os.Exit(1)
	}
}

func printTranslateUsage() {
	fmt.Println("Translate Commands (CQRS + Event Sourcing + Headless AI):")
	fmt.Println("  translate sync           Extract text and generate task files")
	fmt.Println("  translate sync --dry-run Preview extraction without changes")
	fmt.Println("  translate auto <file>    AI translation (headless, requires API key)")
	fmt.Println("  translate apply <file>   Apply translations from task file")
	fmt.Println("  translate apply <file> --dry-run  Preview application")
	fmt.Println("  translate events         View event log (audit trail)")
	fmt.Println()
	fmt.Println("Manual Translation Flow:")
	fmt.Println("  1. mon-tool translate sync                     # Extract text")
	fmt.Println("  2. Edit tasks/translate-th.json manually       # Fill translations")
	fmt.Println("  3. mon-tool translate apply tasks/translate-th.json  # Apply")
	fmt.Println()
	fmt.Println("Headless AI Translation Flow:")
	fmt.Println("  1. export ANTHROPIC_API_KEY=sk-ant-...")
	fmt.Println("  2. mon-tool translate sync                     # Extract text")
	fmt.Println("  3. mon-tool translate auto tasks/translate-th.json   # AI translates")
	fmt.Println("  4. mon-tool translate apply tasks/translate-th.json  # Apply")
	fmt.Println()
	fmt.Println("Examples:")
	fmt.Println("  mon-tool translate sync                        # Extract")
	fmt.Println("  mon-tool translate auto tasks/translate-th.json      # AI translate")
	fmt.Println("  mon-tool translate apply tasks/translate-th.json     # Apply")
	fmt.Println("  mon-tool translate events                      # View history")
}

// handleTranslateSync handles the sync subcommand using CQRS pattern
// VISIBLE CALL FLOW - following ADR 004 + CQRS pattern
func handleTranslateSync(dryRun bool) {
	// Step 1: Get working directory
	rootDir, err := os.Getwd()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error getting current directory: %v\n", err)
		os.Exit(1)
	}

	// Step 2: Load configuration (QUERY - read only)
	config, err := translate.LoadConfig(rootDir)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error loading configuration: %v\n", err)
		os.Exit(1)
	}

	// Step 3: Verify source directory exists
	sourceDir := filepath.Join(rootDir, config.Source.Folder)
	if _, err := os.Stat(sourceDir); os.IsNotExist(err) {
		fmt.Fprintf(os.Stderr, "Error: %s directory not found\n", config.Source.Folder)
		os.Exit(1)
	}

	// Step 4: Create event store (Event Sourcing - Phase 3, path from config)
	eventStore, err := events.NewStore(rootDir, config.Paths.Events)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Warning: failed to create event store: %v\n", err)
		eventStore = nil // Continue without event sourcing
	}
	defer func() {
		if eventStore != nil {
			eventStore.Close()
		}
	}()

	// Step 5: Create command handler with event store
	syncHandler := commands.NewSyncHandler(eventStore)

	// Step 6: Process each target language using COMMANDS
	for _, target := range config.Targets {
		fmt.Printf("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n")
		fmt.Printf("🌐 Syncing %s → %s (%s)\n", config.Source.Language, target.Language, target.Folder)
		fmt.Printf("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n")

		// Step 6a: Create COMMAND object (intent to sync)
		cmd := &commands.SyncCommand{
			RootDir:    rootDir,
			SourceLang: config.Source.Language,
			TargetLang: target.Language,
			DryRun:     dryRun,
		}

		// Step 6b: Execute COMMAND via handler
		fmt.Printf("📂 Scanning %s ...\n", config.Source.Folder)
		result, err := syncHandler.Handle(cmd)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error executing sync command: %v\n", err)
			os.Exit(1)
		}

		// Step 5c: Display results
		fmt.Println()
		if dryRun {
			fmt.Println("🔍 DRY RUN - No changes will be made")
		} else {
			fmt.Printf("✅ Syncing structure for %s...\n", target.Language)
		}
		fmt.Println()
		fmt.Printf("Summary: %d directories created, %d files copied, %d files deleted\n",
			result.DirectoriesCreated, result.FilesCopied, result.FilesDeleted)
		fmt.Println()

		// Step 5d: Show generated tasks
		if len(result.TasksGenerated) > 0 && !dryRun {
			for _, taskFile := range result.TasksGenerated {
				fmt.Printf("✓ Generated %s with translation instructions\n", taskFile)
			}
			fmt.Println()
			fmt.Println("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
			fmt.Println("📝 Translation task ready")
			fmt.Println("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
			fmt.Println()
			fmt.Println("Next steps:")
			fmt.Printf("1. Open %s\n", result.TasksGenerated[0])
			fmt.Println("2. Fill in target_text fields with translations")
			fmt.Printf("3. Run: mon-tool translate apply %s\n", result.TasksGenerated[0])
		}
	}
}

// handleTranslateApply handles the apply subcommand using CQRS pattern
// VISIBLE CALL FLOW - following ADR 004 + CQRS pattern
func handleTranslateApply(taskFile string, dryRun bool) {
	// Step 1: Get working directory
	rootDir, err := os.Getwd()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error getting current directory: %v\n", err)
		os.Exit(1)
	}

	// Step 2: Load configuration (need paths)
	config, err := translate.LoadConfig(rootDir)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error loading configuration: %v\n", err)
		os.Exit(1)
	}

	// Step 3: Create event store (Event Sourcing - Phase 3, path from config)
	eventStore, err := events.NewStore(rootDir, config.Paths.Events)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Warning: failed to create event store: %v\n", err)
		eventStore = nil // Continue without event sourcing
	}
	defer func() {
		if eventStore != nil {
			eventStore.Close()
		}
	}()

	// Step 4: Create COMMAND object (intent to apply translations)
	cmd := &commands.ApplyCommand{
		RootDir:  rootDir,
		TaskFile: taskFile,
		DryRun:   dryRun,
	}

	// Step 5: Create command handler with event store
	applyHandler := commands.NewApplyHandler(eventStore)

	// Step 6: Print header
	fmt.Printf("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n")
	fmt.Printf("🌐 Applying translations from task file\n")
	fmt.Printf("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n")

	if dryRun {
		fmt.Println("🔍 DRY RUN - No changes will be made\n")
	}

	// Step 7: Execute COMMAND via handler
	result, err := applyHandler.Handle(cmd)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error executing apply command: %v\n", err)
		os.Exit(1)
	}

	// Step 8: Display results
	percentage := (result.FilledExtractions * 100) / result.TotalExtractions
	fmt.Printf("📊 Translation Progress: %d/%d (%d%%)\n\n",
		result.FilledExtractions, result.TotalExtractions, percentage)

	if result.FilledExtractions < result.TotalExtractions {
		fmt.Printf("⚠️  Warning: Only %d of %d translations are filled in\n",
			result.FilledExtractions, result.TotalExtractions)
		fmt.Printf("   Partial translations will be applied\n\n")
	}

	if !dryRun {
		fmt.Println()
		fmt.Printf("Summary: %d files processed, %d files skipped\n",
			result.FilesProcessed, result.FilesSkipped)

		if result.TaskFileDeleted {
			fmt.Printf("\n🗑️  Deleted task file: %s\n", taskFile)
			fmt.Println("✅ Translation complete!")
		} else {
			fmt.Printf("\n📝 Task file kept: %s\n", taskFile)
			if result.FilledExtractions < result.TotalExtractions {
				fmt.Println("   (Partial translations remain)")
			} else if result.FilesSkipped > 0 {
				fmt.Println("   (Some files had errors)")
			}
		}
	} else {
		fmt.Println()
		fmt.Println("Summary: Dry-run complete (no files modified)")
	}
}

// handleTranslateEvents displays the event log
// VISIBLE CALL FLOW - Event Sourcing query
func handleTranslateEvents() {
	// Step 1: Get working directory
	rootDir, err := os.Getwd()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error getting current directory: %v\n", err)
		os.Exit(1)
	}

	// Step 2: Load configuration (need events path)
	config, err := translate.LoadConfig(rootDir)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error loading configuration: %v\n", err)
		os.Exit(1)
	}

	// Step 3: Read all events (QUERY - read only, path from config)
	records, err := events.ReadAll(rootDir, config.Paths.Events)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error reading events: %v\n", err)
		os.Exit(1)
	}

	if len(records) == 0 {
		fmt.Println("No events recorded yet.")
		fmt.Println("Run 'translate sync' or 'translate apply' to generate events.")
		return
	}

	// Step 4: Display events
	fmt.Printf("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n")
	fmt.Printf("📜 Translation Event Log (%d events)\n", len(records))
	fmt.Printf("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n")

	// Group by session
	sessions := make(map[string][]events.EventRecord)
	for _, record := range records {
		sessions[record.SessionID] = append(sessions[record.SessionID], record)
	}

	// Display each session
	for sessionID, sessionEvents := range sessions {
		fmt.Printf("Session: %s (%d events)\n", sessionID, len(sessionEvents))
		fmt.Println("─────────────────────────────────────────────")

		for _, record := range sessionEvents {
			timestamp := record.Timestamp.Format("15:04:05")

			switch record.Type {
			case "DirectoryCreated":
				var e events.DirectoryCreated
				if err := record.Unmarshal(&e); err == nil {
					fmt.Printf("[%s] 📁 Created directory: %s\n", timestamp, e.Path)
				}
			case "FileCopied":
				var e events.FileCopied
				if err := record.Unmarshal(&e); err == nil {
					sizeKB := float64(e.Size) / 1024
					fmt.Printf("[%s] 📄 Copied %s → %s (%.1fKB)\n",
						timestamp, filepath.Base(e.SourcePath), filepath.Base(e.TargetPath), sizeKB)
				}
			case "FileDeleted":
				var e events.FileDeleted
				if err := record.Unmarshal(&e); err == nil {
					fmt.Printf("[%s] 🗑️  Deleted: %s (%s)\n", timestamp, e.Path, e.Reason)
				}
			case "TaskGenerated":
				var e events.TaskGenerated
				if err := record.Unmarshal(&e); err == nil {
					fmt.Printf("[%s] ✨ Generated task: %s (%d extractions for %s)\n",
						timestamp, e.TaskFile, e.ExtractionCount, e.TargetLanguage)
				}
			case "TaskLoaded":
				var e events.TaskLoaded
				if err := record.Unmarshal(&e); err == nil {
					fmt.Printf("[%s] 📖 Loaded task: %s (%d/%d translations filled)\n",
						timestamp, e.TaskFile, e.FilledCount, e.ExtractionCount)
				}
			case "TranslationApplied":
				var e events.TranslationApplied
				if err := record.Unmarshal(&e); err == nil {
					fmt.Printf("[%s] ✅ Applied translations: %s (%d applied, %d skipped)\n",
						timestamp, e.FilePath, e.AppliedCount, e.SkippedCount)
				}
			case "TaskDeleted":
				var e events.TaskDeleted
				if err := record.Unmarshal(&e); err == nil {
					fmt.Printf("[%s] 🎉 Completed: %s (task deleted)\n", timestamp, e.TaskFile)
				}
			case "AITranslationStarted":
				var e events.AITranslationStarted
				if err := record.Unmarshal(&e); err == nil {
					fmt.Printf("[%s] 🤖 AI Translation started: %s (%d items, %s)\n",
						timestamp, e.TaskFile, e.ItemsCount, e.Model)
				}
			case "AITranslationCompleted":
				var e events.AITranslationCompleted
				if err := record.Unmarshal(&e); err == nil {
					fmt.Printf("[%s] ✅ AI Translation completed: %d items ($%.4f, %.1fs)\n",
						timestamp, e.ItemsTranslated, e.CostUSD, e.DurationSeconds)
				}
			case "AITranslationFailed":
				var e events.AITranslationFailed
				if err := record.Unmarshal(&e); err == nil {
					fmt.Printf("[%s] ❌ AI Translation failed: %s\n", timestamp, e.Error)
				}
			default:
				fmt.Printf("[%s] %s\n", timestamp, record.Type)
			}
		}
		fmt.Println()
	}

	fmt.Println("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
	fmt.Printf("Total: %d events across %d sessions\n", len(records), len(sessions))
}

// handleTranslateAuto handles the auto subcommand using AI translation (HEADLESS mode)
// VISIBLE CALL FLOW - following ADR 005 Headless AI Translation
func handleTranslateAuto(taskFile string, apiKey string) {
	// Step 1: Get working directory
	rootDir, err := os.Getwd()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error getting current directory: %v\n", err)
		os.Exit(1)
	}

	// Step 2: Load configuration (need events path)
	config, err := translate.LoadConfig(rootDir)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error loading configuration: %v\n", err)
		os.Exit(1)
	}

	// Step 3: Create event store (path from config)
	eventStore, err := events.NewStore(rootDir, config.Paths.Events)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Warning: failed to create event store: %v\n", err)
		eventStore = nil
	}
	defer func() {
		if eventStore != nil {
			eventStore.Close()
		}
	}()

	// Step 4: Print header
	fmt.Printf("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n")
	fmt.Printf("🤖 AI Translation (Headless Mode)\n")
	fmt.Printf("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n")

	// Step 5: Check API key
	if apiKey == "" {
		fmt.Fprintf(os.Stderr, "Error: ANTHROPIC_API_KEY not set\n\n")
		fmt.Println("Set your API key:")
		fmt.Println("  export ANTHROPIC_API_KEY=sk-ant-...")
		fmt.Println("Or pass it directly:")
		fmt.Println("  mon-tool translate auto tasks/translate-th.json --api-key=sk-ant-...")
		os.Exit(1)
	}

	// Step 6: Create AI translator
	translator := ai.NewClaudeTranslator(apiKey, "")
	fmt.Printf("Using model: %s\n", translator.Name())
	fmt.Printf("Task file: %s\n\n", taskFile)

	// Emit AITranslationStarted event
	startTime := time.Now()
	if eventStore != nil {
		// We'll get the item count after loading the task
		eventStore.Append(&events.AITranslationStarted{
			BaseEvent: events.BaseEvent{
				Type:      "AITranslationStarted",
				Occurred:  startTime,
				SessionID: eventStore.SessionID(),
			},
			TaskFile: taskFile,
			Model:    translator.Name(),
		})
	}

	// Step 7: Call AutoTranslate (HEADLESS - no human interaction)
	fmt.Println("🔄 Calling Claude API...")
	task, response, err := translate.AutoTranslate(rootDir, taskFile, translator)
	if err != nil {
		// Emit AITranslationFailed event
		if eventStore != nil {
			eventStore.Append(&events.AITranslationFailed{
				BaseEvent: events.BaseEvent{
					Type:      "AITranslationFailed",
					Occurred:  time.Now(),
					SessionID: eventStore.SessionID(),
				},
				TaskFile: taskFile,
				Error:    err.Error(),
				Model:    translator.Name(),
			})
		}
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}

	duration := time.Since(startTime).Seconds()

	// Step 8: Save updated task file
	if err := translate.SaveTask(rootDir, taskFile, task); err != nil {
		fmt.Fprintf(os.Stderr, "Error saving task file: %v\n", err)
		os.Exit(1)
	}

	// Emit AITranslationCompleted event
	if eventStore != nil {
		eventStore.Append(&events.AITranslationCompleted{
			BaseEvent: events.BaseEvent{
				Type:      "AITranslationCompleted",
				Occurred:  time.Now(),
				SessionID: eventStore.SessionID(),
			},
			TaskFile:        taskFile,
			ItemsTranslated: response.ItemsProcessed,
			InputTokens:     response.Usage.InputTokens,
			OutputTokens:    response.Usage.OutputTokens,
			CostUSD:         response.Usage.EstimatedCost,
			DurationSeconds: duration,
			Model:           translator.Name(),
		})
	}

	// Step 9: Display results
	fmt.Printf("✅ Translation completed!\n\n")
	fmt.Printf("📊 Statistics:\n")
	fmt.Printf("  Items translated: %d\n", response.ItemsProcessed)
	fmt.Printf("  Input tokens:     %d\n", response.Usage.InputTokens)
	fmt.Printf("  Output tokens:    %d\n", response.Usage.OutputTokens)
	fmt.Printf("  Total tokens:     %d\n", response.Usage.TotalTokens)
	fmt.Printf("  Duration:         %.2fs\n", duration)
	fmt.Printf("  Cost:             $%.4f\n\n", response.Usage.EstimatedCost)

	fmt.Printf("✓ Updated task file: %s\n\n", taskFile)
	fmt.Println("Next step:")
	fmt.Printf("  mon-tool translate apply %s\n", taskFile)
}
