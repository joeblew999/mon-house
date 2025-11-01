package translate

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
)

// LoadConfig loads and parses the translate.json configuration file
// Single entry point for configuration loading
func LoadConfig(rootDir string) (*Config, error) {
	configPath := filepath.Join(rootDir, "code", "translate.json")

	data, err := os.ReadFile(configPath)
	if err != nil {
		return nil, fmt.Errorf("failed to read translate.json: %w", err)
	}

	var config Config
	if err := json.Unmarshal(data, &config); err != nil {
		return nil, fmt.Errorf("failed to parse translate.json: %w", err)
	}

	// Set default paths if not specified (NO HARDCODED PATHS!)
	if config.Paths.Tasks == "" {
		config.Paths.Tasks = "tasks"
	}
	if config.Paths.Events == "" {
		config.Paths.Events = ".mon-tool"
	}

	return &config, nil
}
