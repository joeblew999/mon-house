package translate

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

// ScanSource scans the source directory and builds a list of actions for syncing
// Single entry point for scanning and planning sync operations
func ScanSource(rootDir string, sourceDir string, target TargetConfig) ([]SyncAction, error) {
	targetDir := filepath.Join(rootDir, target.Folder)
	var actions []SyncAction

	// Phase 1: Scan source directory and plan copy operations
	err := filepath.Walk(sourceDir, func(sourcePath string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		// Skip the source root directory itself
		if sourcePath == sourceDir {
			return nil
		}

		// Get relative path from source root
		relPath, err := filepath.Rel(sourceDir, sourcePath)
		if err != nil {
			return err
		}

		// Apply rename rules for target
		targetRelPath := applyRenameRules(relPath, target.RenameRules)
		targetPath := filepath.Join(targetDir, targetRelPath)

		if info.IsDir() {
			// Create directory in target
			actions = append(actions, SyncAction{
				Action: "mkdir",
				Target: targetPath,
			})
		} else {
			// Determine file type
			fileType := determineFileType(sourcePath)

			// Copy file
			actions = append(actions, SyncAction{
				Action: "copy",
				Source: sourcePath,
				Target: targetPath,
				Type:   fileType,
			})
		}

		return nil
	})

	if err != nil {
		return nil, fmt.Errorf("error scanning source directory: %w", err)
	}

	// Phase 2: Scan target directory and plan delete operations
	if _, err := os.Stat(targetDir); !os.IsNotExist(err) {
		err = filepath.Walk(targetDir, func(targetPath string, info os.FileInfo, err error) error {
			if err != nil {
				return err
			}

			// Skip the target root directory itself
			if targetPath == targetDir {
				return nil
			}

			// Get relative path from target root
			relPath, err := filepath.Rel(targetDir, targetPath)
			if err != nil {
				return err
			}

			// Reverse rename rules to find source path
			sourceRelPath := reverseRenameRules(relPath, target.RenameRules)
			sourcePath := filepath.Join(sourceDir, sourceRelPath)

			// Check if source exists
			if _, err := os.Stat(sourcePath); os.IsNotExist(err) {
				// Source doesn't exist, mark for deletion
				actions = append(actions, SyncAction{
					Action: "delete",
					Target: targetPath,
				})
			}

			return nil
		})

		if err != nil {
			return nil, fmt.Errorf("error scanning target directory: %w", err)
		}
	}

	return actions, nil
}

// applyRenameRules applies rename rules to a relative path
func applyRenameRules(relPath string, renameRules map[string]string) string {
	for oldExt, newExt := range renameRules {
		if strings.HasSuffix(relPath, oldExt) && !strings.HasSuffix(relPath, newExt) {
			return strings.TrimSuffix(relPath, oldExt) + newExt
		}
	}
	return relPath
}

// reverseRenameRules reverses rename rules to find the original path
func reverseRenameRules(relPath string, renameRules map[string]string) string {
	for oldExt, newExt := range renameRules {
		if strings.HasSuffix(relPath, newExt) {
			return strings.TrimSuffix(relPath, newExt) + oldExt
		}
	}
	return relPath
}

// determineFileType determines the type of a file based on its extension
func determineFileType(filePath string) string {
	if strings.HasSuffix(filePath, ".svg") {
		return "svg"
	} else if strings.HasSuffix(filePath, ".md") {
		return "md"
	}
	return "other"
}
