# ADR 004: Visible Call Flow Architecture

## Status
Accepted

## Summary

This is a change to the code.

Make translate so that all pkg code is single entry.

Then have all cmds fully control the call flow.

This makes it easier for me and AI to code.

This makes it easier to control producing tasks for claude.

We do this on the translate code first, and then look at the other code later.

This will mean that all changes to the system as like tasks, which is perfect for later making into a CQRS (Command Query Responsibility Segregation) system.

## Context

When writing code, especially with AI assistance, it becomes extremely difficult to understand "what is happening" because:

- **Call flow is hidden**: Functions call other functions, logic spreads across multiple files
- **Hard to debug**: Can't see the sequence of operations, hard to know where errors originate
- **Hard to reason about**: Must trace through many files to understand one operation
- **Hard to modify**: Changing behavior requires understanding scattered implementation details
- **Hard for AI**: AI loses track of context when logic is distributed across files
- **Hard to collaborate**: Humans can't quickly understand what an operation does

### The Core Problem: Invisible Call Flow

When business logic is mixed with orchestration, you lose visibility of **what happens when**:

```go
// Current problem: Can't see the flow
func handleSomeOperation() {
    // What does this do? Have to read the function...
    result := doSomething()

    // What about this? More jumping around...
    processResult(result)

    // Wait, what's the full sequence again?
}
```

This makes refactoring nearly impossible because you can't see what you're changing.

## Decision

Implement the **"Single Entry Point at cmd/ Level"** pattern:

### Architecture Pattern

**cmd/** = **Orchestration layer** with visible, sequential call flow
- Contains the complete sequence of operations for each command
- Shows "what happens when" as readable, top-to-bottom code
- Each step is ONE call to a pkg/ function
- Easy to read, easy to modify, easy to debug

**pkg/** = **Business logic** as simple, single-entry functions
- Each function has ONE clear purpose
- Each function is a "single entry point" for one operation
- Returns results to cmd/ for the next step
- No complex call chains between pkg/ functions

### The Pattern

```go
// cmd/translate.go - THE FLOW IS VISIBLE HERE

func handleTranslateSync(dryRun bool) {
    // Step 1: Get working directory
    rootDir, err := os.Getwd()
    if err != nil {
        fmt.Fprintf(os.Stderr, "Error: %v\n", err)
        os.Exit(1)
    }

    // Step 2: Load configuration (single entry to pkg)
    config, err := translate.LoadConfig(rootDir)
    if err != nil {
        fmt.Fprintf(os.Stderr, "Error loading config: %v\n", err)
        os.Exit(1)
    }

    // Step 3: Scan source files (single entry to pkg)
    sourceFiles, err := translate.ScanSource(config.Source)
    if err != nil {
        fmt.Fprintf(os.Stderr, "Error scanning source: %v\n", err)
        os.Exit(1)
    }

    // Step 4: Plan sync actions (single entry to pkg)
    actions, err := translate.PlanSync(sourceFiles, config.Targets)
    if err != nil {
        fmt.Fprintf(os.Stderr, "Error planning sync: %v\n", err)
        os.Exit(1)
    }

    // Step 5: Execute sync (single entry to pkg)
    if !dryRun {
        err = translate.ExecuteSync(actions)
        if err != nil {
            fmt.Fprintf(os.Stderr, "Error executing sync: %v\n", err)
            os.Exit(1)
        }
    }

    // Step 6: Extract translatable text (single entry to pkg)
    extractions, err := translate.ExtractText(sourceFiles)
    if err != nil {
        fmt.Fprintf(os.Stderr, "Error extracting text: %v\n", err)
        os.Exit(1)
    }

    // Step 7: Generate task JSON files (single entry to pkg)
    err = translate.GenerateTasks(extractions, config.Targets, rootDir)
    if err != nil {
        fmt.Fprintf(os.Stderr, "Error generating tasks: %v\n", err)
        os.Exit(1)
    }

    fmt.Println("✅ Translation sync complete")
}
```

Each pkg/ function is simple and focused:

```go
// pkg/translate/config.go
func LoadConfig(rootDir string) (*Config, error) {
    // Single entry point: Load and parse translate.json
    // Returns: parsed config or error
    // That's it. Nothing else.
}

// pkg/translate/scan.go
func ScanSource(sourceConfig SourceConfig) ([]File, error) {
    // Single entry point: Scan source directory for files
    // Returns: list of files or error
    // That's it. Nothing else.
}
```

### What "Single Entry" Means

**Single entry** = Each operation has ONE function that does that operation and returns

**NOT this** (multiple scattered functions):
```go
func scanFiles() { ... }
func filterFiles() { ... }
func validateFiles() { ... }
// Caller has to know to call all three in the right order
```

**YES this** (single entry point):
```go
func ScanSource(config SourceConfig) ([]File, error) {
    // This ONE function handles everything needed for scanning
    files := scanFiles()
    files = filterFiles(files)
    files = validateFiles(files)
    return files, nil
}
// Caller just calls this ONE function
```

## Implementation

### Guidelines for New Code

When writing NEW commands or functionality, follow this pattern:

**For cmd/ files**:
- CLI parsing (flags, arguments)
- Help/usage text  
- Orchestration (visible call flow, step-by-step)
- Error handling and display
- Exit codes

**For pkg/ files**:
- Business logic functions (single entry point per operation)
- File I/O operations
- JSON parsing/generation
- Data transformations
- Validation logic

### Pattern Structure

```
code/mon-tool/
├── cmd/
│   └── mycommand.go          # Orchestration with visible flow
└── pkg/
    └── mypackage/
        ├── types.go          # Data structures
        ├── operation1.go     # Single-entry function
        ├── operation2.go     # Single-entry function
        └── operation3.go     # Single-entry function
```

### Example: Writing a New Command

```go
// cmd/mycommand.go - Visible call flow

func handleMyCommand(args []string) {
    // Step 1: Parse arguments
    config := parseArgs(args)
    
    // Step 2: Load data (single entry to pkg)
    data, err := mypackage.LoadData(config.Source)
    handleError(err)
    
    // Step 3: Process data (single entry to pkg)
    result, err := mypackage.ProcessData(data, config.Options)
    handleError(err)
    
    // Step 4: Save result (single entry to pkg)
    err = mypackage.SaveResult(result, config.Output)
    handleError(err)
    
    fmt.Println("✅ Complete")
}

// pkg/mypackage/process.go - Single entry point

func ProcessData(data Data, options Options) (*Result, error) {
    // This ONE function does all the processing
    // Internal helper functions can exist but are private
    validated := validateData(data)
    transformed := transformData(validated, options)
    result := buildResult(transformed)
    return result, nil
}
```

### Guidelines

**What stays in cmd/**:
- CLI parsing (flags, arguments)
- Help/usage text
- Orchestration (the visible call flow)
- Error handling and display
- Exit codes

**What moves to pkg/**:
- File I/O operations
- JSON parsing/generation
- Business logic and algorithms
- Data transformations
- Validation logic

## Consequences

### Positive

✅ **Visibility**: The complete operation flow is readable in one place
✅ **Easier debugging**: Add logging between steps, know exactly where you are
✅ **Easier refactoring**: Change the sequence by moving lines in cmd/
✅ **Easier for AI**: AI can see the full flow and understand the operation
✅ **Easier for humans**: No jumping between files to understand behavior
✅ **Easier to modify**: Insert, remove, or reorder steps trivially
✅ **Better testing**: Mock pkg/ functions, test the orchestration
✅ **Task-oriented**: The flow IS the task specification
✅ **CQRS foundation**: Each operation becomes a clear command with visible execution

### Trade-offs

⚖️ **More files**: Split between cmd/ and pkg/ means more files to navigate
⚖️ **Discipline required**: Must maintain the pattern, resist putting logic in cmd/
⚖️ **Initial refactoring**: Requires upfront work to restructure existing code

### Neutral

⚪ **File organization**: pkg/ structure can grow over time as needed
⚪ **Error handling**: Each step needs explicit error handling (verbosity vs. clarity)

## Examples

### Before: Hidden Flow

```go
// Current cmd/translate.go - can't see what's happening
func handleTranslateSync(dryRun bool) {
    rootDir, err := os.Getwd()
    if err != nil {
        fmt.Fprintf(os.Stderr, "Error: %v\n", err)
        os.Exit(1)
    }

    config, err := loadTranslateConfig(rootDir)
    if err != nil { ... }

    // What does syncLanguage do? Have to read 200 lines...
    for _, target := range config.Targets {
        syncLanguage(rootDir, sourceDir, target, config, dryRun)
    }
}

// syncLanguage has everything mixed together:
// - file scanning
// - copying logic
// - text extraction
// - JSON generation
// All in one 200-line function
```

### After: Visible Flow

```go
// New cmd/translate.go - the complete flow is visible

func handleTranslateSync(dryRun bool) {
    // Step 1: Get working directory
    rootDir, err := os.Getwd()
    handleError(err)

    // Step 2: Load configuration
    config, err := translate.LoadConfig(rootDir)
    handleError(err)

    // Step 3: Scan source files
    sourceFiles, err := translate.ScanSource(config.Source)
    handleError(err)

    // Step 4: For each target language
    for _, target := range config.Targets {
        fmt.Printf("\n🌐 Syncing %s → %s\n", config.Source.Language, target.Language)

        // Step 4a: Plan sync actions
        actions, err := translate.PlanSync(sourceFiles, target)
        handleError(err)

        // Step 4b: Execute sync
        if !dryRun {
            err = translate.ExecuteSync(rootDir, actions)
            handleError(err)
        }

        // Step 4c: Extract translatable text
        extractions, err := translate.ExtractText(rootDir, actions)
        handleError(err)

        // Step 4d: Generate task JSON
        err = translate.GenerateTask(rootDir, target, extractions)
        handleError(err)
    }

    fmt.Println("\n✅ Translation sync complete")
}

// Each pkg function is simple and focused:
// pkg/translate/scan.go
func ScanSource(config SourceConfig) ([]File, error) {
    // Just scan and return files
}

// pkg/translate/sync.go
func PlanSync(sourceFiles []File, target TargetConfig) ([]Action, error) {
    // Just determine what to copy/delete
}

// pkg/translate/sync.go
func ExecuteSync(rootDir string, actions []Action) error {
    // Just execute the file operations
}

// pkg/translate/extract.go
func ExtractText(rootDir string, actions []Action) ([]Extraction, error) {
    // Just extract text from files
}

// pkg/translate/task.go
func GenerateTask(rootDir string, target TargetConfig, extractions []Extraction) error {
    // Just generate task JSON file
}
```

## Benefits for Refactoring

With visible call flow, refactoring becomes trivial:

### Adding a validation step
```go
// Just insert one line
config, err := translate.LoadConfig(rootDir)
handleError(err)

err = translate.ValidateConfig(config)  // ← Added one line
handleError(err)

sourceFiles, err := translate.ScanSource(config.Source)
```

### Adding logging
```go
// Just add prints between steps
actions, err := translate.PlanSync(sourceFiles, target)
handleError(err)
fmt.Printf("Planned %d actions\n", len(actions))  // ← Added

err = translate.ExecuteSync(rootDir, actions)
handleError(err)
fmt.Println("Sync complete")  // ← Added
```

### Skipping a step conditionally
```go
// Just add an if statement
if config.EnableExtraction {  // ← Added
    extractions, err := translate.ExtractText(rootDir, actions)
    handleError(err)
}  // ← Added
```

### Reordering steps
```go
// Just move lines around - the flow is explicit
```

## CQRS Architecture Integration

This architecture pattern is designed to evolve into full CQRS (Command Query Responsibility Segregation):

### CQRS Principles

**Command Query Separation:**
- **Commands**: Operations that CHANGE state (write, delete, modify)
- **Queries**: Operations that READ state (no side effects, return data)
- **Rule**: A function is either a Command OR a Query, never both

### Current Implementation (Partial CQRS)

**Queries (pure read operations):**
```go
// pkg/translate/config.go
func LoadConfig(rootDir string) (*Config, error)  // Query: reads config

// pkg/translate/scan.go
func ScanSource(...) ([]SyncAction, error)  // Query: scans and returns plan

// pkg/translate/sync.go
func GetSyncStats(actions) (int, int, int)  // Query: calculates stats
func GetTranslatableFiles(actions) []string  // Query: filters list
```

**Commands (state-changing operations):**
```go
// pkg/translate/sync.go
func ExecuteSync(actions []SyncAction) error  // Command: modifies filesystem

// pkg/translate/task.go
func GenerateTask(...) error  // Command: creates file

// pkg/translate/apply.go
func ApplyTranslations(...) (*Stats, error)  // Command: modifies files
func DeleteTask(...) error  // Command: deletes file
```

### Evolution to Full CQRS

The architecture sets up for future CQRS enhancements:

**Phase 1: Current State (ADR 004 compliance)**
- ✅ Visible call flow in cmd/
- ✅ Single-entry functions in pkg/
- ✅ Separation of read (scan) from write (execute)
- ⚠️ Commands return errors (not pure command pattern)

**Phase 2: Command Objects (Future)**
```go
// pkg/translate/commands/types.go
type SyncCommand struct {
    RootDir   string
    TargetLang string
    DryRun    bool
}

type ApplyCommand struct {
    RootDir  string
    TaskFile string
    DryRun   bool
}
```

**Phase 3: Event Sourcing (Future)**
```go
// pkg/translate/events/types.go
type FileCopied struct {
    Timestamp time.Time
    Source    string
    Target    string
}

type TranslationApplied struct {
    Timestamp time.Time
    File      string
    Count     int
}

// Event store persists all state changes
type EventStore interface {
    Append(event Event) error
    Query(filter EventFilter) ([]Event, error)
}
```

**Phase 4: Command Handler (Future)**
```go
// pkg/translate/handlers/sync.go
type SyncHandler struct {
    eventStore EventStore
}

func (h *SyncHandler) Handle(cmd SyncCommand) error {
    // Execute command
    // Emit events
    // Store events
}
```

**Phase 5: Read Models (Future)**
```go
// Separate read models built from event store
type TranslationStats struct {
    TotalFiles      int
    TranslatedFiles int
    LastSyncTime    time.Time
}

// Query side reads from optimized read models
func GetTranslationStats(lang string) (*TranslationStats, error)
```

### Benefits of CQRS Evolution

✅ **Audit trail**: Event store provides complete history of all changes
✅ **Replay capability**: Rebuild state by replaying events
✅ **Time travel**: Query system state at any point in history
✅ **Separation of concerns**: Write model optimized for commands, read model for queries
✅ **Scalability**: Read and write sides can scale independently

### Why Start with Visible Call Flow

The visible call flow pattern (ADR 004) is the foundation for CQRS because:

1. **Clear boundaries**: Each operation is already a discrete unit
2. **Easy to wrap**: Convert `ExecuteSync(actions)` → `SyncHandler.Handle(SyncCommand)`
3. **Observable flow**: Easy to insert event emission between steps
4. **Testable**: Mock event store, test command handlers independently

The current architecture makes CQRS adoption incremental - no big rewrite needed.

## Related

- **CQRS pattern**: Command Query Responsibility Segregation
- **Event Sourcing**: Store state changes as events
- **Task-oriented architecture**: The visible flow IS the task specification
- **Hexagonal architecture**: pkg/ is the domain, cmd/ is the adapter
- **Domain-Driven Design**: Commands and events model business operations

## Future Enhancements

- Add event sourcing with event store
- Implement command and query objects
- Add command handlers that emit events
- Build read models from event streams
- Add timing/performance measurement between steps
- Add transaction-like rollback on errors
- Parallel execution of independent steps
- Progress reporting for long-running operations

---

**Date**: 2025-10-31
**Author**: Claude (AI Assistant) with user guidance
**Reviewers**: [To be added]
