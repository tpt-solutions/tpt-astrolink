// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

package scheduler

import (
	"testing"
	"time"

	"github.com/tptsolutions/tpt-astrolink/relay/internal/registry"
)

func TestScheduleAssignsHighPriorityFirst(t *testing.T) {
	reg := registry.New()
	reg.Register(registry.Node{ID: "n1", Available: true, Lat: 10, Lon: 10})
	reg.Register(registry.Node{ID: "n2", Available: true, Lat: 20, Lon: 20})

	targets := []Target{
		{ID: "low", Priority: 1},
		{ID: "high", Priority: 10},
	}

	// Force night so isNight passes regardless of wall clock.
	now := time.Date(2026, 7, 20, 22, 0, 0, 0, time.UTC)
	got := Schedule(reg.Available(), targets, now)

	if len(got) != 2 {
		t.Fatalf("expected 2 assignments, got %d", len(got))
	}
	if got[0].TargetID != "high" {
		t.Errorf("expected high-priority target first, got %q", got[0].TargetID)
	}
}

func TestScheduleSkipsUnavailableNodes(t *testing.T) {
	reg := registry.New()
	reg.Register(registry.Node{ID: "off", Available: false})

	targets := []Target{{ID: "t1", Priority: 5}}
	now := time.Date(2026, 7, 20, 22, 0, 0, 0, time.UTC)
	if got := Schedule(reg.Available(), targets, now); len(got) != 0 {
		t.Fatalf("expected no assignments for unavailable nodes, got %d", len(got))
	}
}

func TestScheduleOneTargetPerNode(t *testing.T) {
	reg := registry.New()
	reg.Register(registry.Node{ID: "only", Available: true})

	targets := []Target{
		{ID: "a", Priority: 9},
		{ID: "b", Priority: 8},
	}
	now := time.Date(2026, 7, 20, 22, 0, 0, 0, time.UTC)
	got := Schedule(reg.Available(), targets, now)
	if len(got) != 1 {
		t.Fatalf("expected single assignment (one node), got %d", len(got))
	}
}
