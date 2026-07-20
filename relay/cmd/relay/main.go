// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

// Command tpt-relay is the entrypoint for the Relay crowdsourcing scheduler.
package main

import (
	"log"
	"time"

	"github.com/tptsolutions/tpt-astrolink/relay/internal/registry"
	"github.com/tptsolutions/tpt-astrolink/relay/internal/scheduler"
)

func main() {
	reg := registry.New()
	reg.Register(registry.Node{ID: "node-a", Lat: -36.8, Lon: 174.7, Timezone: "Pacific/Auckland", Available: true})
	reg.Register(registry.Node{ID: "node-b", Lat: 37.7, Lon: -122.4, Timezone: "America/Los_Angeles", Available: true})

	targets := []scheduler.Target{
		{ID: "m31", RA: 10.68, Dec: 41.26, Priority: 10},
		{ID: "ngc253", RA: 11.89, Dec: -25.29, Priority: 5},
	}

	assignments := scheduler.Schedule(reg.Available(), targets, time.Now())
	log.Printf("relay scheduled %d assignments", len(assignments))
	for _, a := range assignments {
		log.Printf("  %s -> %s", a.TargetID, a.NodeID)
	}
}
