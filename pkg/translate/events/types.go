package events

import "time"

// Event is the base interface for all events
type Event interface {
	EventType() string
	Timestamp() time.Time
}

// BaseEvent contains common fields for all events
type BaseEvent struct {
	Type      string    `json:"type"`
	Occurred  time.Time `json:"timestamp"`
	SessionID string    `json:"session_id,omitempty"`
}

func (e BaseEvent) EventType() string {
	return e.Type
}

func (e BaseEvent) Timestamp() time.Time {
	return e.Occurred
}

// Sync Events (from SyncCommand)

// DirectoryCreated fires when a target directory is created
type DirectoryCreated struct {
	BaseEvent
	Path string `json:"path"`
}

// FileCopied fires when a file is copied to target
type FileCopied struct {
	BaseEvent
	SourcePath string `json:"source_path"`
	TargetPath string `json:"target_path"`
	Size       int64  `json:"size_bytes"`
	FileType   string `json:"file_type"` // "svg", "md", "other"
}

// FileDeleted fires when a file is deleted from target
type FileDeleted struct {
	BaseEvent
	Path   string `json:"path"`
	Reason string `json:"reason"` // "not_in_source", etc.
}

// TaskGenerated fires when a translation task file is created
type TaskGenerated struct {
	BaseEvent
	TaskFile       string `json:"task_file"`
	TargetLanguage string `json:"target_language"`
	FileCount      int    `json:"file_count"`
	ExtractionCount int   `json:"extraction_count"`
}

// Apply Events (from ApplyCommand)

// TaskLoaded fires when a task file is loaded
type TaskLoaded struct {
	BaseEvent
	TaskFile        string `json:"task_file"`
	TargetLanguage  string `json:"target_language"`
	FileCount       int    `json:"file_count"`
	ExtractionCount int    `json:"extraction_count"`
	FilledCount     int    `json:"filled_count"`
}

// TranslationApplied fires when translations are applied to a file
type TranslationApplied struct {
	BaseEvent
	FilePath        string `json:"file_path"`
	FileType        string `json:"file_type"`
	AppliedCount    int    `json:"applied_count"`
	SkippedCount    int    `json:"skipped_count"`
}

// TranslationFailed fires when applying translations fails
type TranslationFailed struct {
	BaseEvent
	FilePath string `json:"file_path"`
	FileType string `json:"file_type"`
	Error    string `json:"error"`
}

// TaskDeleted fires when a task file is deleted after successful application
type TaskDeleted struct {
	BaseEvent
	TaskFile string `json:"task_file"`
	Reason   string `json:"reason"` // "completed", "manual", etc.
}

// Query Events (from read operations - optional, can skip for performance)

// ConfigLoaded fires when configuration is loaded
type ConfigLoaded struct {
	BaseEvent
	ConfigPath   string `json:"config_path"`
	SourceLang   string `json:"source_language"`
	TargetCount  int    `json:"target_count"`
}

// TextExtracted fires when text is extracted from a file
type TextExtracted struct {
	BaseEvent
	FilePath        string `json:"file_path"`
	FileType        string `json:"file_type"`
	ExtractionCount int    `json:"extraction_count"`
}

// AI Translation Events

// AITranslationStarted fires when AI translation begins
type AITranslationStarted struct {
	BaseEvent
	TaskFile   string `json:"task_file"`
	ItemsCount int    `json:"items_count"`
	Model      string `json:"model"`
}

// AITranslationCompleted fires when AI translation succeeds
type AITranslationCompleted struct {
	BaseEvent
	TaskFile         string  `json:"task_file"`
	ItemsTranslated  int     `json:"items_translated"`
	InputTokens      int     `json:"input_tokens"`
	OutputTokens     int     `json:"output_tokens"`
	CostUSD          float64 `json:"cost_usd"`
	DurationSeconds  float64 `json:"duration_seconds"`
	Model            string  `json:"model"`
}

// AITranslationFailed fires when AI translation fails
type AITranslationFailed struct {
	BaseEvent
	TaskFile string `json:"task_file"`
	Error    string `json:"error"`
	Model    string `json:"model"`
}
