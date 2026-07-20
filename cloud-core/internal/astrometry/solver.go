// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

// Package astrometry wraps the Astrometry.net C-backend (via CLI/API) for
// plate-solving uploaded FITS frames. See docs/architecture.md (Data flow).
package astrometry

import (
	"context"
	"os/exec"
)

// Result holds plate-solve outputs written back to Postgres.
type Result struct {
	RACenter   float64 `json:"ra_center"`
	DecCenter  float64 `json:"dec_center"`
	FOVW       float64 `json:"fov_w_deg"`
	FOVH       float64 `json:"fov_h_deg"`
	Orientation float64 `json:"orientation_deg"`
}

// Solver runs the Astrometry.net CLI against a local FITS file.
type Solver struct {
	Binary string // path to `solve-field`
}

func (s *Solver) Solve(ctx context.Context, fitsPath string) (*Result, error) {
	// TODO(Phase 3): invoke solve-field, parse WCS, build Result.
	cmd := exec.CommandContext(ctx, s.Binary, fitsPath)
	if err := cmd.Run(); err != nil {
		return nil, err
	}
	return &Result{}, nil
}
