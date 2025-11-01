package translate

import (
	"fmt"
	"io"
	"os"
	"path/filepath"
)

// ExecuteSync executes a list of sync actions (mkdir, copy, delete)
// Single entry point for executing file operations
func ExecuteSync(actions []SyncAction) error {
	for _, action := range actions {
		switch action.Action {
		case "mkdir":
			if err := os.MkdirAll(action.Target, 0755); err != nil {
				return fmt.Errorf("failed to create directory %s: %w", action.Target, err)
			}
		case "copy":
			if err := copyFile(action.Source, action.Target); err != nil {
				return fmt.Errorf("failed to copy %s to %s: %w", action.Source, action.Target, err)
			}
		case "delete":
			if err := os.RemoveAll(action.Target); err != nil {
				return fmt.Errorf("failed to delete %s: %w", action.Target, err)
			}
		}
	}
	return nil
}

// copyFile copies a file from src to dst
func copyFile(src, dst string) error {
	// Ensure parent directory exists
	dstDir := filepath.Dir(dst)
	if err := os.MkdirAll(dstDir, 0755); err != nil {
		return fmt.Errorf("failed to create parent directory: %w", err)
	}

	// Open source file
	srcFile, err := os.Open(src)
	if err != nil {
		return err
	}
	defer srcFile.Close()

	// Create destination file
	dstFile, err := os.Create(dst)
	if err != nil {
		return err
	}
	defer dstFile.Close()

	// Copy contents
	_, err = io.Copy(dstFile, srcFile)
	if err != nil {
		return err
	}

	// Sync to disk
	return dstFile.Sync()
}

// GetSyncStats calculates statistics from a list of sync actions
func GetSyncStats(actions []SyncAction) (mkdirs, copies, deletes int) {
	for _, action := range actions {
		switch action.Action {
		case "mkdir":
			mkdirs++
		case "copy":
			copies++
		case "delete":
			deletes++
		}
	}
	return
}

// GetTranslatableFiles returns a list of files that need translation from actions
func GetTranslatableFiles(actions []SyncAction) []string {
	var files []string
	for _, action := range actions {
		if action.Action == "copy" && (action.Type == "svg" || action.Type == "md") {
			files = append(files, action.Target)
		}
	}
	return files
}
