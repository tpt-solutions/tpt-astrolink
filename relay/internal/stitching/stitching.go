// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

// Package stitching combines multi-node observations in the cloud into a
// unified dataset. Phase 5: a simple circular-mean co-add of centers; a real
// implementation would register/stack the FITS pixels.
package stitching

import (
	"context"
	"fmt"
	"math"
)

// Observation is a reference to a node's captured frame.
type Observation struct {
	NodeID    string
	ObjectKey string
	RACenter  float64 // degrees
	DecCenter float64 // degrees
}

// Result describes the stitched product.
type Result struct {
	ObjectKey  string
	RACenter   float64
	DecCenter  float64
	FrameCount int
}

// Stitch co-adds observation centers into a single stacked product reference.
func Stitch(ctx context.Context, obs []Observation) (*Result, error) {
	if len(obs) == 0 {
		return nil, fmt.Errorf("stitching: no observations")
	}
	// Circular mean of RA (degrees) to avoid the 359->0 wrap-around problem.
	var x, y, decSum float64
	for _, o := range obs {
		rad := o.RACenter * math.Pi / 180
		x += math.Cos(rad)
		y += math.Sin(rad)
		decSum += o.DecCenter
	}
	deg := 180 / math.Pi
	ra := math.Atan2(y, x) * deg
	ra = math.Mod(ra+360, 360)
	dec := decSum / float64(len(obs))

	return &Result{
		ObjectKey:  fmt.Sprintf("s3://tpt-astrolink-fits/stitched/%d-frames.fits", len(obs)),
		RACenter:   ra,
		DecCenter:  dec,
		FrameCount: len(obs),
	}, nil
}
