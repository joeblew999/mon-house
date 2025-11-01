package ai

// TranslationRequest represents a batch translation request to AI
type TranslationRequest struct {
	SourceLanguage string              `json:"source_language"`
	TargetLanguage string              `json:"target_language"`
	LanguageName   string              `json:"language_name"`
	Domain         string              `json:"domain"` // "architecture", "medical", etc.
	Terminology    map[string]string   `json:"terminology,omitempty"`
	Notes          []string            `json:"notes,omitempty"`
	Items          []TranslationItem   `json:"items"`
}

// TranslationItem represents a single text to translate
type TranslationItem struct {
	ID          string `json:"id"`           // Unique ID for tracking
	Context     string `json:"context"`      // "heading", "paragraph", "label"
	SourceText  string `json:"source_text"`
	TargetText  string `json:"target_text"`  // Filled by AI
}

// TranslationResponse represents the AI's translation response
type TranslationResponse struct {
	Success        bool              `json:"success"`
	ItemsProcessed int               `json:"items_processed"`
	Translations   []TranslationItem `json:"translations"`
	Error          string            `json:"error,omitempty"`
	Usage          Usage             `json:"usage"`
}

// Usage tracks API usage/costs
type Usage struct {
	InputTokens  int     `json:"input_tokens"`
	OutputTokens int     `json:"output_tokens"`
	TotalTokens  int     `json:"total_tokens"`
	EstimatedCost float64 `json:"estimated_cost_usd"`
}

// Translator is the interface for AI translation services
type Translator interface {
	// Translate performs batch translation
	Translate(req *TranslationRequest) (*TranslationResponse, error)

	// Name returns the translator name (e.g., "claude-3-5-sonnet")
	Name() string
}
