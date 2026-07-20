// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

package gateway_test

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/gorilla/websocket"
	"github.com/tptsolutions/tpt-astrolink/cloud-core/internal/gateway"
	"github.com/tptsolutions/tpt-astrolink/cloud-core/internal/protocol"
)

// fakeBridge records commands forwarded from the gateway and lets tests drive
// broadcasts back to connected clients.
type fakeBridge struct {
	received chan protocol.Envelope
}

func (f *fakeBridge) SendCommand(env protocol.Envelope) error {
	f.received <- env
	return nil
}

func newTestServer(t *testing.T) (*httptest.Server, *fakeBridge, *gateway.Hub) {
	t.Helper()
	bridge := &fakeBridge{received: make(chan protocol.Envelope, 8)}
	hub := gateway.NewHub(bridge)
	go hub.Run()
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		hub.ServeWS(w, r)
	}))
	t.Cleanup(srv.Close)
	return srv, bridge, hub
}

func dial(t *testing.T, url string) *websocket.Conn {
	t.Helper()
	c, _, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		t.Fatalf("dial: %v", err)
	}
	t.Cleanup(func() { c.Close() })
	return c
}

// TestWSCommandRoutedToBridge verifies a client command envelope is forwarded
// to the MQTT bridge exactly once, preserving type/id/nodeId/payload.
func TestWSCommandRoutedToBridge(t *testing.T) {
	srv, bridge, _ := newTestServer(t)
	c := dial(t, "ws"+strings.TrimPrefix(srv.URL, "http"))

	cmd := protocol.Envelope{
		Type:   protocol.CmdSlew,
		ID:     "cmd-1",
		NodeID: "node-A",
		Payload: mustJSON(t, protocol.Coords{RA: 12.5, Dec: -30, Epoch: "J2000"}),
	}
	if err := c.WriteJSON(cmd); err != nil {
		t.Fatalf("write: %v", err)
	}

	select {
	case got := <-bridge.received:
		if got.Type != cmd.Type || got.ID != cmd.ID || got.NodeID != cmd.NodeID {
			t.Fatalf("command mismatch: %+v", got)
		}
		var c2 protocol.Coords
		if err := json.Unmarshal(got.Payload, &c2); err != nil {
			t.Fatalf("payload unmarshal: %v", err)
		}
		if c2.RA != 12.5 || c2.Dec != -30 {
			t.Fatalf("coords mismatch: %+v", c2)
		}
	case <-time.After(2 * time.Second):
		t.Fatal("command not forwarded to bridge")
	}
}

// TestWSBroadcastDeliversToSubscribers verifies telemetry/events broadcast by
// the hub are delivered to all connected clients.
func TestWSBroadcastDeliversToSubscribers(t *testing.T) {
	srv, _, hub := newTestServer(t)
	c1 := dial(t, "ws"+strings.TrimPrefix(srv.URL, "http"))
	c2 := dial(t, "ws"+strings.TrimPrefix(srv.URL, "http"))

	env := protocol.Envelope{
		Type:   protocol.TelemetryMount,
		NodeID: "node-A",
		Payload: mustJSON(t, map[string]any{
			"ra": 1.0, "dec": 2.0, "status": "tracking",
		}),
	}
	hub.Broadcast(env)

	for _, c := range []*websocket.Conn{c1, c2} {
		c.SetReadDeadline(time.Now().Add(2 * time.Second))
		var got protocol.Envelope
		if err := c.ReadJSON(&got); err != nil {
			t.Fatalf("read: %v", err)
		}
		if got.Type != protocol.TelemetryMount {
			t.Fatalf("unexpected type %q", got.Type)
		}
	}
}

// TestWSNonCommandIgnored verifies non-command client messages are not routed
// to the bridge (e.g. stray telemetry from a misbehaving client).
func TestWSNonCommandIgnored(t *testing.T) {
	srv, bridge, _ := newTestServer(t)
	c := dial(t, "ws"+strings.TrimPrefix(srv.URL, "http"))

	stray := protocol.Envelope{Type: protocol.TelemetryWeather, NodeID: "node-A"}
	if err := c.WriteJSON(stray); err != nil {
		t.Fatalf("write: %v", err)
	}
	select {
	case <-bridge.received:
		t.Fatal("non-command message was routed to bridge")
	case <-time.After(300 * time.Millisecond):
	}
}

func mustJSON(t *testing.T, v any) json.RawMessage {
	t.Helper()
	b, err := json.Marshal(v)
	if err != nil {
		t.Fatal(err)
	}
	return b
}
