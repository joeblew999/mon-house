package config

import (
	"encoding/json"
	"os"
)

// DrawingsConfig represents the drawings.json structure
type DrawingsConfig struct {
	Drawings struct {
		Version     string `json:"version"`
		Description string `json:"description"`
		BasePath    string `json:"basePath"`
		Scale       struct {
			Unit            string `json:"unit"`
			PixelsPerMeter  int    `json:"pixelsPerMeter"`
			Description     string `json:"description"`
		} `json:"scale"`
		PaperSize struct {
			Format      string `json:"format"`
			WidthMM     int    `json:"widthMM"`
			HeightMM    int    `json:"heightMM"`
			Orientation string `json:"orientation"`
		} `json:"paperSize"`
		Legend struct {
			Items []struct {
				Text  string `json:"text"`
				Color string `json:"color"`
			} `json:"items"`
		} `json:"legend"`
		Files []DrawingFile `json:"files"`
	} `json:"drawings"`
}

// DrawingFile represents a single drawing file entry
type DrawingFile struct {
	Path      string `json:"path"`
	Type      string `json:"type"`
	Status    string `json:"status"`
	Width     int    `json:"width"`
	Height    int    `json:"height"`
	ViewBox   string `json:"viewBox"`
	Title     string `json:"title"`
	Subtitle  string `json:"subtitle"`
	ScaleText string `json:"scaleText"`
}

// LoadDrawingsConfig reads and parses drawings.json
func LoadDrawingsConfig(path string) (*DrawingsConfig, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}

	var config DrawingsConfig
	if err := json.Unmarshal(data, &config); err != nil {
		return nil, err
	}

	return &config, nil
}

// LoadJSON reads and parses any JSON file into interface{}
func LoadJSON(path string) (interface{}, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}

	var result interface{}
	if err := json.Unmarshal(data, &result); err != nil {
		return nil, err
	}

	return result, nil
}
