// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

// Package mqttbridge bridges Cloud Core and Edge Agents over MQTT 5.0.
// Topic layout: docs/protocols/mqtt-contract.md.
package mqttbridge

import (
	"encoding/json"
	"log"

	"github.com/tptsolutions/tpt-astrolink/cloud-core/internal/protocol"
)

// Bridge publishes commands to nodes and subscribes to their telemetry/events.
type Bridge struct {
	publish func(topic string, payload []byte) error
	hub     TelemetrySink
}

// TelemetrySink delivers inbound telemetry/events to the WebSocket hub.
type TelemetrySink interface {
	Broadcast(env protocol.Envelope) error
}

// Publisher publishes a payload to an MQTT topic.
type Publisher interface {
	Publish(topic string, payload []byte) error
}

// NewBridge creates a bridge. The sink may be set later via SetSink to break
// the hub<->bridge construction cycle.
func NewBridge(publish func(string, []byte) error, hub TelemetrySink) *Bridge {
	return &Bridge{publish: publish, hub: hub}
}

// SetSink installs the telemetry sink (used to break the construction cycle
// with the WebSocket hub).
func (b *Bridge) SetSink(hub TelemetrySink) { b.hub = hub }

// SendCommand forwards a client command envelope to the target node.
func (b *Bridge) SendCommand(env protocol.Envelope) error {
	topic := "tpt/v1/" + env.NodeID + "/cmd/all"
	payload, err := json.Marshal(env)
	if err != nil {
		return err
	}
	return b.publish(topic, payload)
}

// OnMessage handles an inbound publish from an Edge Agent and re-publishes it
// to the WebSocket hub.
func (b *Bridge) OnMessage(topic string, payload []byte) {
	var env protocol.Envelope
	if err := json.Unmarshal(payload, &env); err != nil {
		log.Printf("mqtt parse: %v", err)
		return
	}
	env.NodeID = nodeIDFromTopic(topic)
	if b.hub != nil {
		_ = b.hub.Broadcast(env)
	}
}

// nodeIDFromTopic extracts the node id from `tpt/v1/<nodeId>/...`.
func nodeIDFromTopic(topic string) string {
	const prefix = "tpt/v1/"
	if len(topic) <= len(prefix) {
		return ""
	}
	rest := topic[len(prefix):]
	for i := 0; i < len(rest); i++ {
		if rest[i] == '/' {
			return rest[:i]
		}
	}
	return rest
}

