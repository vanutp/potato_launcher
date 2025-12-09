package api

import (
	"fmt"
)

type ErrorCode string

const (
	ErrCodeValidation ErrorCode = "validation_error"
	ErrCodeConflict   ErrorCode = "conflict"
	ErrCodeNotFound   ErrorCode = "not_found"
)

type AppError struct {
	Code    ErrorCode
	Message string
	Field   string
}

func (e *AppError) Error() string {
	if e.Field != "" {
		return fmt.Sprintf("%s: %s", e.Field, e.Message)
	}
	return e.Message
}

func NewValidationError(field, message string) *AppError {
	return &AppError{
		Code:    ErrCodeValidation,
		Message: message,
		Field:   field,
	}
}

func NewConflictError(message string) *AppError {
	return &AppError{
		Code:    ErrCodeConflict,
		Message: message,
	}
}

func NewNotFoundError(message string) *AppError {
	return &AppError{
		Code:    ErrCodeNotFound,
		Message: message,
	}
}
