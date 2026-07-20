// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

// Package metrics exposes Prometheus counters for Cloud Core observability.
package metrics

import "github.com/prometheus/client_golang/prometheus"

var (
	// CommandsReceived counts inbound client commands.
	CommandsReceived = prometheus.NewCounterVec(prometheus.CounterOpts{
		Namespace: "tpt",
		Subsystem: "cloud_core",
		Name:      "commands_received_total",
		Help:      "Total client commands received.",
	}, []string{"type"})

	// TelemetryPublished counts telemetry/events forwarded to clients.
	TelemetryPublished = prometheus.NewCounter(prometheus.CounterOpts{
		Namespace: "tpt",
		Subsystem: "cloud_core",
		Name:      "telemetry_published_total",
		Help:      "Total telemetry/event envelopes published to clients.",
	})
)

func init() {
	prometheus.MustRegister(CommandsReceived, TelemetryPublished)
}
