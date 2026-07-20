// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

// Package auth provides session issuing/validation for multi-user access.
// TODO(Phase 3): wire to users table + JWT.
package auth

import (
	"errors"
	"time"
)

// Session represents an authenticated user session.
type Session struct {
	UserID    string
	ExpiresAt time.Time
}

var errUnauthorized = errors.New("unauthorized")

// Validate checks a bearer token and returns the session, or an error.
func Validate(token string) (*Session, error) {
	if token == "" {
		return nil, errUnauthorized
	}
	// TODO(Phase 3): verify JWT signature & expiry.
	return &Session{UserID: "placeholder", ExpiresAt: time.Now().Add(time.Hour)}, nil
}
