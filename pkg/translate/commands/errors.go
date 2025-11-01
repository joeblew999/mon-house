package commands

import "errors"

// Command validation errors
var (
	ErrEmptyRootDir    = errors.New("root directory cannot be empty")
	ErrEmptySourceLang = errors.New("source language cannot be empty")
	ErrEmptyTargetLang = errors.New("target language cannot be empty")
	ErrEmptyTaskFile   = errors.New("task file path cannot be empty")
)
