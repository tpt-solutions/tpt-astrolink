// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

// Package registry tracks Edge Node availability/registration for the Relay
// scheduling engine. In-memory for now; swap for Postgres (cloud-core) later.
package registry

import (
	"sync"
	"time"
)

// Node is a registered Edge Agent.
type Node struct {
	ID        string
	Lat       float64
	Lon       float64
	Timezone  string
	Available bool
	LastSeen  time.Time
}

// Registry stores node registrations and their current availability.
type Registry struct {
	mu    sync.RWMutex
	nodes map[string]Node
}

func New() *Registry {
	return &Registry{nodes: make(map[string]Node)}
}

func (r *Registry) Register(n Node) {
	r.mu.Lock()
	defer r.mu.Unlock()
	n.LastSeen = time.Now()
	r.nodes[n.ID] = n
}

func (r *Registry) SetAvailable(id string, available bool) {
	r.mu.Lock()
	defer r.mu.Unlock()
	if n, ok := r.nodes[id]; ok {
		n.Available = available
		n.LastSeen = time.Now()
		r.nodes[id] = n
	}
}

func (r *Registry) Available() []Node {
	r.mu.RLock()
	defer r.mu.RUnlock()
	out := make([]Node, 0, len(r.nodes))
	for _, n := range r.nodes {
		if n.Available {
			out = append(out, n)
		}
	}
	return out
}

func (r *Registry) Get(id string) (Node, bool) {
	r.mu.RLock()
	defer r.mu.RUnlock()
	n, ok := r.nodes[id]
	return n, ok
}
