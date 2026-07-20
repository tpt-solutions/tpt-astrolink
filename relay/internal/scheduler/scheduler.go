// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

// Package scheduler implements the "Relay" crowdsourcing engine: it assigns
// observation targets to Edge Nodes based on local night-time availability and
// resolves assignment conflicts. See docs/architecture.md (Relay flow).
package scheduler

import (
	"sort"
	"time"

	"github.com/tptsolutions/tpt-astrolink/relay/internal/registry"
)

// Node is an alias for the registered Edge Agent type.
type Node = registry.Node

// Target is an observation target to be scheduled.
type Target struct {
	ID    string
	RA    float64
	Dec   float64
	Priority int
}

// Assignment binds a target to a node for a window.
type Assignment struct {
	NodeID   string
	TargetID string
	Start    time.Time
	End      time.Time
}

// Schedule assigns each target to the best available node. Conflict resolution:
// higher-priority targets win; a node takes at most one target per night window.
func Schedule(nodes []Node, targets []Target, now time.Time) []Assignment {
	avail := make(map[string]Node)
	for _, n := range nodes {
		if n.Available {
			avail[n.ID] = n
		}
	}

	sorted := append([]Target(nil), targets...)
	sort.SliceStable(sorted, func(i, j int) bool {
		return sorted[i].Priority > sorted[j].Priority
	})

	var out []Assignment
	busy := make(map[string]bool)
	for _, t := range sorted {
		for _, n := range avail {
			if busy[n.ID] {
				continue
			}
			if !isNight(n, now) {
				continue
			}
			busy[n.ID] = true
			out = append(out, Assignment{
				NodeID:   n.ID,
				TargetID: t.ID,
				Start:    now,
				End:      now.Add(6 * time.Hour),
			})
			break
		}
	}
	return out
}

// isNight is a placeholder for local night-time determination by latitude.
func isNight(n Node, now time.Time) bool {
	// TODO(Phase 5): compute sun altitude from lat/lon + date.
	return now.Hour() >= 19 || now.Hour() < 6
}
