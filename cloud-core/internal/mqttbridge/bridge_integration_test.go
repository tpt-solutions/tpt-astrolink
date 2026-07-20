// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

package mqttbridge

import (
	"encoding/json"
	"testing"

	"github.com/tptsolutions/tpt-astrolink/cloud-core/internal/protocol"
)

// fakeSink records broadcast envelopes for contract assertions.
type fakeSink struct {
	got []protocol.Envelope
}

func (s *fakeSink) Broadcast(env protocol.Envelope) error {
	s.got = append(s.got, env)
	return nil
}

func mustJSON(t *testing.T, v any) json.RawMessage {
	t.Helper()
	b, err := json.Marshal(v)
	if err != nil {
		t.Fatal(err)
	}
	return b
}

// TestTelemetryContractEndToEnd exercises the inbound MQTT contract: a real
// telemetry publish payload (as an Edge Agent would send it) is delivered via
// OnMessage and must be re-broadcast to the WebSocket hub with the correct
// node id extracted from the topic.
func TestTelemetryContractEndToEnd(t *testing.T) {
	sink := &fakeSink{}
	b := NewBridge(func(string, []byte) error { return nil }, sink)

	payload, _ := json.Marshal(protocol.Envelope{
		Type:   protocol.TelemetryMount,
		ID:     "t-1",
		Payload: mustJSON(t, map[string]any{
			"ra": 12.5, "dec": -30.0, "alt": 0, "az": 0, "tracking": true, "status": "tracking",
		}),
	})

	b.OnMessage("tpt/v1/node-7/tele/mount", payload)

	if len(sink.got) != 1 {
		t.Fatalf("expected 1 broadcast, got %d", len(sink.got))
	}
	env := sink.got[0]
	if env.Type != protocol.TelemetryMount {
		t.Errorf("unexpected type %q", env.Type)
	}
	if env.NodeID != "node-7" {
		t.Errorf("node id not extracted from topic: %q", env.NodeID)
	}
	var m map[string]any
	if err := json.Unmarshal(env.Payload, &m); err != nil {
		t.Fatal(err)
	}
	if m["ra"].(float64) != 12.5 {
		t.Errorf("ra not preserved: %v", m["ra"])
	}
}

// TestEventContract verifies an edge ToO event flows through OnMessage.
func TestEventContract(t *testing.T) {
	sink := &fakeSink{}
	b := NewBridge(func(string, []byte) error { return nil }, sink)

	payload, _ := json.Marshal(protocol.Envelope{
		Type:   protocol.AlertToo,
		ID:     "evt-1",
		Payload: mustJSON(t, map[string]any{
			"objectId": "obj-9", "ra": 1.0, "dec": 2.0, "magDelta": 0.5, "confidence": 0.9, "imageKey": "k",
		}),
	})
	b.OnMessage("tpt/v1/node-7/evt/too", payload)

	if len(sink.got) != 1 {
		t.Fatalf("expected 1 broadcast, got %d", len(sink.got))
	}
	if sink.got[0].Type != protocol.AlertToo {
		t.Errorf("unexpected type %q", sink.got[0].Type)
	}
}
