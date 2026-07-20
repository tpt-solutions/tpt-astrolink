// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

// Package worker is the cloud worker triggered on S3 FITS upload: it runs
// astrometry and writes RA/Dec metadata to Postgres. See docs/architecture.md.
package worker

import (
	"context"
	"log"

	"github.com/tptsolutions/tpt-astrolink/cloud-core/internal/astrometry"
	"github.com/tptsolutions/tpt-astrolink/cloud-core/internal/postgres"
)

// Worker processes observation jobs.
type Worker struct {
	solver *astrometry.Solver
	store  *postgres.Store
}

func New(solver *astrometry.Solver, store *postgres.Store) *Worker {
	return &Worker{solver: solver, store: store}
}

// Process downloads/points at a FITS file, plate-solves it, and records
// metadata. fitsPath is the local path to the uploaded frame.
func (w *Worker) Process(ctx context.Context, observationID, fitsPath string) error {
	res, err := w.solver.Solve(ctx, fitsPath)
	if err != nil {
		return err
	}
	log.Printf("solved observation=%s ra=%f dec=%f", observationID, res.RACenter, res.DecCenter)
	// TODO(Phase 3): write metadata row via store.
	return nil
}
