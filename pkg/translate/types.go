package translate

// Config represents the translate.json configuration
type Config struct {
	Source struct {
		Language string `json:"language"`
		Folder   string `json:"folder"`
	} `json:"source"`
	Targets []TargetConfig `json:"targets"`
	FileTypes struct {
		Translatable []string `json:"translatable"`
		CopyOnly     []string `json:"copy_only"`
	} `json:"file_types"`
	Paths struct {
		Tasks  string `json:"tasks"`  // Default: "tasks"
		Events string `json:"events"` // Default: ".mon-tool"
	} `json:"paths"`
}

// TargetConfig represents a translation target language configuration
type TargetConfig struct {
	Language         string            `json:"language"`
	LanguageName     string            `json:"language_name"`
	Folder           string            `json:"folder"`
	RenameRules      map[string]string `json:"rename_rules"`
	TranslationNotes []string          `json:"translation_notes"`
}

// SyncAction represents a file operation action
type SyncAction struct {
	Action string // "mkdir", "copy", "delete"
	Source string // Source file path (for copy)
	Target string // Target file/directory path
	Type   string // "svg", "md", "other" (for copy)
}

// TextExtraction represents a single text element that needs translation
type TextExtraction struct {
	Line       int    `json:"line"`
	XPath      string `json:"xpath,omitempty"`      // For SVG/XML
	Context    string `json:"context,omitempty"`    // For Markdown (e.g., "heading", "paragraph")
	SourceText string `json:"source_text"`
	TargetText string `json:"target_text"`
}

// TaskFile represents a file that needs translation in a task
type TaskFile struct {
	Source      string           `json:"source"`
	Target      string           `json:"target"`
	Type        string           `json:"type"`
	Extractions []TextExtraction `json:"extractions,omitempty"`
}

// Task represents a translation task
type Task struct {
	Task             string              `json:"task"`
	SourceLanguage   string              `json:"source_language"`
	TargetLanguage   string              `json:"target_language"`
	LanguageName     string              `json:"language_name"`
	Files            []TaskFile          `json:"files"`
	TranslationNotes []string            `json:"translation_notes"`
	Instructions     map[string][]string `json:"instructions"`
}

// ApplyStats represents statistics from applying translations
type ApplyStats struct {
	TotalExtractions  int
	FilledExtractions int
	FilesProcessed    int
	FilesSkipped      int
}
