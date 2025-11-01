package events

import (
	"bufio"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sync"
	"time"

	"github.com/google/uuid"
)

// Store is an append-only event store
// Events are written as JSON lines to a file
type Store struct {
	filePath  string
	sessionID string
	mu        sync.Mutex
	file      *os.File
}

// NewStore creates a new event store
// Events are stored in: {rootDir}/{eventsPath}/events.jsonl
// eventsPath comes from config (not hardcoded!)
func NewStore(rootDir string, eventsPath string) (*Store, error) {
	eventsDir := filepath.Join(rootDir, eventsPath)
	if err := os.MkdirAll(eventsDir, 0755); err != nil {
		return nil, fmt.Errorf("failed to create events directory: %w", err)
	}

	filePath := filepath.Join(eventsDir, "events.jsonl")
	file, err := os.OpenFile(filePath, os.O_CREATE|os.O_APPEND|os.O_WRONLY, 0644)
	if err != nil {
		return nil, fmt.Errorf("failed to open event store: %w", err)
	}

	// Generate session ID for grouping related events
	sessionID := uuid.New().String()[:8]

	return &Store{
		filePath:  filePath,
		sessionID: sessionID,
		file:      file,
	}, nil
}

// Append writes an event to the store
func (s *Store) Append(event Event) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	// Convert event to JSON
	data, err := json.Marshal(event)
	if err != nil {
		return fmt.Errorf("failed to marshal event: %w", err)
	}

	// Write as JSON line
	if _, err := s.file.Write(append(data, '\n')); err != nil {
		return fmt.Errorf("failed to write event: %w", err)
	}

	return nil
}

// Close closes the event store
func (s *Store) Close() error {
	s.mu.Lock()
	defer s.mu.Unlock()

	if s.file != nil {
		return s.file.Close()
	}
	return nil
}

// SessionID returns the current session ID
func (s *Store) SessionID() string {
	return s.sessionID
}

// EventRecord represents a stored event with metadata
type EventRecord struct {
	Raw       json.RawMessage `json:"raw"`
	Type      string          `json:"type"`
	Timestamp time.Time       `json:"timestamp"`
	SessionID string          `json:"session_id"`
}

// Unmarshal deserializes the raw event data into a specific event type
func (r *EventRecord) Unmarshal(v interface{}) error {
	return json.Unmarshal(r.Raw, v)
}

// ReadAll reads all events from the store
func ReadAll(rootDir string, eventsPath string) ([]EventRecord, error) {
	filePath := filepath.Join(rootDir, eventsPath, "events.jsonl")

	file, err := os.Open(filePath)
	if err != nil {
		if os.IsNotExist(err) {
			return []EventRecord{}, nil // No events yet
		}
		return nil, fmt.Errorf("failed to open event store: %w", err)
	}
	defer file.Close()

	var records []EventRecord
	scanner := bufio.NewScanner(file)

	for scanner.Scan() {
		var record EventRecord
		line := scanner.Bytes()

		// First unmarshal to get type and timestamp
		var base BaseEvent
		if err := json.Unmarshal(line, &base); err != nil {
			continue // Skip malformed lines
		}

		record.Raw = line
		record.Type = base.Type
		record.Timestamp = base.Occurred
		record.SessionID = base.SessionID

		records = append(records, record)
	}

	if err := scanner.Err(); err != nil {
		return nil, fmt.Errorf("failed to read events: %w", err)
	}

	return records, nil
}

// ReadSession reads all events for a specific session
func ReadSession(rootDir string, eventsPath string, sessionID string) ([]EventRecord, error) {
	all, err := ReadAll(rootDir, eventsPath)
	if err != nil {
		return nil, err
	}

	var session []EventRecord
	for _, record := range all {
		if record.SessionID == sessionID {
			session = append(session, record)
		}
	}

	return session, nil
}

// ReadSince reads all events since a timestamp
func ReadSince(rootDir string, eventsPath string, since time.Time) ([]EventRecord, error) {
	all, err := ReadAll(rootDir, eventsPath)
	if err != nil {
		return nil, err
	}

	var filtered []EventRecord
	for _, record := range all {
		if record.Timestamp.After(since) || record.Timestamp.Equal(since) {
			filtered = append(filtered, record)
		}
	}

	return filtered, nil
}

// Clear removes all events (use with caution!)
func Clear(rootDir string, eventsPath string) error {
	filePath := filepath.Join(rootDir, eventsPath, "events.jsonl")
	if err := os.Remove(filePath); err != nil && !os.IsNotExist(err) {
		return fmt.Errorf("failed to clear events: %w", err)
	}
	return nil
}
