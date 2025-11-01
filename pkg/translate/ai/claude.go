package ai

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"strings"
)

// ClaudeTranslator implements the Translator interface using Anthropic's Claude API
type ClaudeTranslator struct {
	APIKey string
	Model  string // "claude-3-5-sonnet-20241022", etc.
	client *http.Client
}

// NewClaudeTranslator creates a new Claude translator
func NewClaudeTranslator(apiKey string, model string) *ClaudeTranslator {
	if apiKey == "" {
		apiKey = os.Getenv("ANTHROPIC_API_KEY")
	}
	if model == "" {
		model = "claude-3-5-sonnet-20241022"
	}
	return &ClaudeTranslator{
		APIKey: apiKey,
		Model:  model,
		client: &http.Client{},
	}
}

// Name returns the translator name
func (c *ClaudeTranslator) Name() string {
	return c.Model
}

// Translate performs batch translation using Claude API
func (c *ClaudeTranslator) Translate(req *TranslationRequest) (*TranslationResponse, error) {
	if c.APIKey == "" {
		return nil, fmt.Errorf("Claude API key not set (use ANTHROPIC_API_KEY env var)")
	}

	// Build the prompt
	prompt := c.buildPrompt(req)

	// Call Claude API
	apiReq := map[string]interface{}{
		"model": c.Model,
		"max_tokens": 4096,
		"messages": []map[string]string{
			{
				"role": "user",
				"content": prompt,
			},
		},
	}

	jsonData, err := json.Marshal(apiReq)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal API request: %w", err)
	}

	httpReq, err := http.NewRequest("POST", "https://api.anthropic.com/v1/messages", bytes.NewBuffer(jsonData))
	if err != nil {
		return nil, fmt.Errorf("failed to create HTTP request: %w", err)
	}

	httpReq.Header.Set("Content-Type", "application/json")
	httpReq.Header.Set("x-api-key", c.APIKey)
	httpReq.Header.Set("anthropic-version", "2023-06-01")

	resp, err := c.client.Do(httpReq)
	if err != nil {
		return nil, fmt.Errorf("failed to call Claude API: %w", err)
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to read API response: %w", err)
	}

	if resp.StatusCode != 200 {
		return nil, fmt.Errorf("Claude API error (status %d): %s", resp.StatusCode, string(body))
	}

	// Parse Claude response
	var apiResp struct {
		Content []struct {
			Text string `json:"text"`
		} `json:"content"`
		Usage struct {
			InputTokens  int `json:"input_tokens"`
			OutputTokens int `json:"output_tokens"`
		} `json:"usage"`
	}

	if err := json.Unmarshal(body, &apiResp); err != nil {
		return nil, fmt.Errorf("failed to parse API response: %w", err)
	}

	if len(apiResp.Content) == 0 {
		return nil, fmt.Errorf("no content in API response")
	}

	// Parse translations from Claude's response
	translations, err := c.parseTranslations(apiResp.Content[0].Text, req.Items)
	if err != nil {
		return nil, fmt.Errorf("failed to parse translations: %w", err)
	}

	// Calculate estimated cost (rough estimate)
	inputCost := float64(apiResp.Usage.InputTokens) * 0.000003   // $3 per 1M input tokens
	outputCost := float64(apiResp.Usage.OutputTokens) * 0.000015 // $15 per 1M output tokens
	totalCost := inputCost + outputCost

	return &TranslationResponse{
		Success:        true,
		ItemsProcessed: len(translations),
		Translations:   translations,
		Usage: Usage{
			InputTokens:   apiResp.Usage.InputTokens,
			OutputTokens:  apiResp.Usage.OutputTokens,
			TotalTokens:   apiResp.Usage.InputTokens + apiResp.Usage.OutputTokens,
			EstimatedCost: totalCost,
		},
	}, nil
}

// buildPrompt constructs the translation prompt for Claude
func (c *ClaudeTranslator) buildPrompt(req *TranslationRequest) string {
	var sb strings.Builder

	sb.WriteString(fmt.Sprintf("You are a professional translator specializing in %s.\n\n", req.Domain))
	sb.WriteString(fmt.Sprintf("Translate the following text from %s (%s) to %s (%s).\n\n",
		req.SourceLanguage, req.SourceLanguage, req.TargetLanguage, req.LanguageName))

	if len(req.Terminology) > 0 {
		sb.WriteString("TERMINOLOGY (use these exact translations):\n")
		for en, target := range req.Terminology {
			sb.WriteString(fmt.Sprintf("- %s → %s\n", en, target))
		}
		sb.WriteString("\n")
	}

	if len(req.Notes) > 0 {
		sb.WriteString("TRANSLATION NOTES:\n")
		for _, note := range req.Notes {
			sb.WriteString(fmt.Sprintf("- %s\n", note))
		}
		sb.WriteString("\n")
	}

	sb.WriteString("TEXTS TO TRANSLATE:\n")
	sb.WriteString("Return a JSON array with the translations in this exact format:\n")
	sb.WriteString("[\n")
	sb.WriteString("  {\"id\": \"ID_HERE\", \"target_text\": \"TRANSLATION_HERE\"},\n")
	sb.WriteString("  ...\n")
	sb.WriteString("]\n\n")

	sb.WriteString("Items:\n")
	for _, item := range req.Items {
		sb.WriteString(fmt.Sprintf("ID: %s\n", item.ID))
		sb.WriteString(fmt.Sprintf("Context: %s\n", item.Context))
		sb.WriteString(fmt.Sprintf("Source: %s\n", item.SourceText))
		sb.WriteString("\n")
	}

	return sb.String()
}

// parseTranslations extracts translations from Claude's response
func (c *ClaudeTranslator) parseTranslations(response string, originalItems []TranslationItem) ([]TranslationItem, error) {
	// Find JSON array in response
	startIdx := strings.Index(response, "[")
	endIdx := strings.LastIndex(response, "]")

	if startIdx == -1 || endIdx == -1 {
		return nil, fmt.Errorf("no JSON array found in response")
	}

	jsonStr := response[startIdx : endIdx+1]

	var parsed []struct {
		ID         string `json:"id"`
		TargetText string `json:"target_text"`
	}

	if err := json.Unmarshal([]byte(jsonStr), &parsed); err != nil {
		return nil, fmt.Errorf("failed to parse JSON: %w", err)
	}

	// Match translations back to original items by ID
	idMap := make(map[string]string)
	for _, p := range parsed {
		idMap[p.ID] = p.TargetText
	}

	result := make([]TranslationItem, len(originalItems))
	for i, item := range originalItems {
		result[i] = item
		if translation, ok := idMap[item.ID]; ok {
			result[i].TargetText = translation
		}
	}

	return result, nil
}
