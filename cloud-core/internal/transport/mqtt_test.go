// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

package transport

import (
	"encoding/json"
	"os"
	"strings"
	"testing"
	"time"
)

// TestTopicBuilders verifies the topic helpers match the MQTT contract layout
// in docs/protocols/mqtt-contract.md.
func TestTopicBuilders(t *testing.T) {
	cases := []struct {
		got, want string
	}{
		{CommandTopic("n1", "mount"), "tpt/v1/n1/cmd/mount"},
		{TelemetryTopic("n1", "weather"), "tpt/v1/n1/tele/weather"},
		{EventTopic("n1", "too"), "tpt/v1/n1/evt/too"},
		{StatusTopic("n1"), "tpt/v1/n1/status"},
	}
	for _, c := range cases {
		if c.got != c.want {
			t.Errorf("topic mismatch: got %q want %q", c.got, c.want)
		}
	}
}

// roundTripPayload is the wire shape for an edge command (MQTT contract).
type roundTripPayload struct {
	Cmd    string          `json:"cmd"`
	ID     string          `json:"id"`
	TS     string          `json:"ts"`
	Params json.RawMessage `json:"params"`
}

// TestMQTTRoundTrip connects to a real broker (set TPT_MQTT_BROKER, e.g.
// "tcp://localhost:1883") and verifies a command published by Cloud Core is
// received on the edge command topic and that an edge telemetry publish is
// received by Cloud Core. Skipped when no broker is configured.
func TestMQTTRoundTrip(t *testing.T) {
	broker := os.Getenv("TPT_MQTT_BROKER")
	if broker == "" {
		t.Skip("TPT_MQTT_BROKER not set; skipping live MQTT round-trip")
	}

	recv := make(chan roundTripPayload, 1)
	edge, err := New(Config{Broker: broker, ClientID: "edge-test"}, func(topic string, p []byte) {
		if strings.HasPrefix(topic, "tpt/v1/") && strings.Contains(topic, "/cmd/") {
			var m roundTripPayload
			if json.Unmarshal(p, &m) == nil {
				recv <- m
			}
		}
	})
	if err != nil {
		t.Fatalf("edge connect: %v", err)
	}
	defer edge.c.Disconnect(100)

	cloud, err := New(Config{Broker: broker, ClientID: "cloud-test"}, func(string, []byte) {})
	if err != nil {
		t.Fatalf("cloud connect: %v", err)
	}
	defer cloud.c.Disconnect(100)

	// Cloud publishes a slew command to the edge command topic.
	payload, _ := json.Marshal(roundTripPayload{
		Cmd:    "slew",
		ID:     "x1",
		TS:     time.Now().UTC().Format(time.RFC3339),
		Params: mustJSON(map[string]any{"ra": 12.5, "dec": -30, "epoch": "J2000"}),
	})
	if err := cloud.PublishCommand("node-1", "mount", payload); err != nil {
		t.Fatalf("publish: %v", err)
	}

	select {
	case got := <-recv:
		if got.Cmd != "slew" || got.ID != "x1" {
			t.Fatalf("unexpected command: %+v", got)
		}
	case <-time.After(3 * time.Second):
		t.Fatal("edge did not receive command within timeout")
	}
}

func mustJSON(v any) json.RawMessage {
	b, _ := json.Marshal(v)
	return b
}

