// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

package stitching

import (
	"context"
	"math"
	"testing"
)

func TestStitchCircularMeanRA(t *testing.T) {
	obs := []Observation{
		{RACenter: 359, DecCenter: 0},
		{RACenter: 1, DecCenter: 0},
	}
	r, err := Stitch(context.Background(), obs)
	if err != nil {
		t.Fatal(err)
	}
	if math.Abs(r.RACenter-0) > 1e-6 {
		t.Errorf("expected RA ~0 (circular mean of 359 and 1), got %f", r.RACenter)
	}
	if r.FrameCount != 2 {
		t.Errorf("expected 2 frames, got %d", r.FrameCount)
	}
}

func TestStitchEmpty(t *testing.T) {
	if _, err := Stitch(context.Background(), nil); err == nil {
		t.Fatal("expected error for empty observations")
	}
}
