package commands

// Command is the interface that all commands must implement
// Commands are operations that CHANGE state (CQRS pattern)
type Command interface {
	// Validate checks if the command is valid before execution
	Validate() error
}

// SyncCommand represents a request to sync translations from source to target
// This is a COMMAND (changes filesystem state)
type SyncCommand struct {
	RootDir    string // Working directory
	SourceLang string // Source language code (e.g., "en")
	TargetLang string // Target language code (e.g., "th")
	DryRun     bool   // If true, preview only without making changes
}

// Validate checks if the SyncCommand is valid
func (c *SyncCommand) Validate() error {
	if c.RootDir == "" {
		return ErrEmptyRootDir
	}
	if c.SourceLang == "" {
		return ErrEmptySourceLang
	}
	if c.TargetLang == "" {
		return ErrEmptyTargetLang
	}
	return nil
}

// ApplyCommand represents a request to apply translations from a task file
// This is a COMMAND (changes filesystem state)
type ApplyCommand struct {
	RootDir  string // Working directory
	TaskFile string // Path to task JSON file (e.g., "tasks/translate-th.json")
	DryRun   bool   // If true, preview only without making changes
}

// Validate checks if the ApplyCommand is valid
func (c *ApplyCommand) Validate() error {
	if c.RootDir == "" {
		return ErrEmptyRootDir
	}
	if c.TaskFile == "" {
		return ErrEmptyTaskFile
	}
	return nil
}

// Result represents the outcome of executing a command
// This separates the command (intent) from the result (outcome)
type Result struct {
	Success bool
	Message string
	Data    interface{} // Optional data payload
}

// SyncResult contains the outcome of a SyncCommand
type SyncResult struct {
	DirectoriesCreated int
	FilesCopied        int
	FilesDeleted       int
	TasksGenerated     []string // List of task files generated
}

// ApplyResult contains the outcome of an ApplyCommand
type ApplyResult struct {
	FilesProcessed    int
	FilesSkipped      int
	TotalExtractions  int
	FilledExtractions int
	TaskFileDeleted   bool
}
