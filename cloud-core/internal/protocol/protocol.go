// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

// Package protocol defines the shared message envelope used across the
// Cloud Core WebSocket and MQTT boundaries. Field shapes mirror
// docs/protocols/websocket-contract.md and docs/protocols/mqtt-contract.md.
package protocol

import "encoding/json"

// Envelope is the common JSON wrapper for all client<->server messages.
type Envelope struct {
	Type   string          `json:"type"`
	ID     string          `json:"id"`
	TS     string          `json:"ts"`
	NodeID string          `json:"nodeId,omitempty"`
	Payload json.RawMessage `json:"payload,omitempty"`
}

// Ack is returned for every command, paired by Envelope.ID.
type Ack struct {
	OK    bool   `json:"ok"`
	Error string `json:"error,omitempty"`
}

// Command types (client -> cloud).
const (
	CmdSlew         = "cmd.slew"
	CmdSlewStop     = "cmd.slewStop"
	CmdFocus        = "cmd.focus"
	CmdFocusRel     = "cmd.focusRelative"
	CmdWeather      = "cmd.weather.refresh"
	CmdImagingStart = "cmd.imaging.start"
	CmdImagingStop  = "cmd.imaging.stop"
	CmdSubscribe    = "cmd.subscribe"
)

// Server -> client message types.
const (
	MsgAck              = "ack"
	TelemetryMount      = "telemetry.mount"
	TelemetryFocuser    = "telemetry.focuser"
	TelemetryWeather    = "telemetry.weather"
	EventImagingProg    = "event.imaging.progress"
	AlertToo            = "alert.too"
	EventAstrometry     = "event.astrometry"
)

// Coordinate command payload (slew).
type Coords struct {
	RA    float64 `json:"ra"`
	Dec   float64 `json:"dec"`
	Epoch string  `json:"epoch"`
}
