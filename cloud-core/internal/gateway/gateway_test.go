// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

package gateway

import (
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/gorilla/websocket"
	"github.com/tptsolutions/tpt-astrolink/cloud-core/internal/protocol"
)

// fakeBridge records commands forwarded from the gateway to the MQTT bridge.
type fakeBridge struct {
	got []protocol.Envelope
}

func (f *fakeBridge) SendCommand(env protocol.Envelope) error {
	f.got = append(f.got, env)
	return nil
}

// TestWebSocketRoundTrip verifies a client can connect, send a command
// envelope, and the gateway forwards it to the bridge (command flow) while
// also delivering a server-originated telemetry envelope back to the client
// (telemetry flow).
func TestWebSocketRoundTrip(t *testing.T) {
	bridge := &fakeBridge{}
	hub := NewHub(bridge)
	go hub.Run()

	srv := httptest.NewServer(http.HandlerFunc(hub.ServeWS))
	defer srv.Close()

	wsURL := "ws" + strings.TrimPrefix(srv.URL, "http")
	conn, _, err := websocket.DefaultDialer.Dial(wsURL, nil)
	if err != nil {
		t.Fatalf("dial: %v", err)
	}
	defer conn.Close()

	// Client sends a command.
	cmd := protocol.Envelope{Type: protocol.CmdSlew, ID: "c1", NodeID: "n1", Payload: []byte(`{"ra":1,"dec":2}`)}
	if err := conn.WriteJSON(cmd); err != nil {
		t.Fatalf("write: %v", err)
	}

	// Gateway should forward it to the bridge.
	deadline := time.Now().Add(2 * time.Second)
	for time.Now().Before(deadline) {
		if len(bridge.got) > 0 {
			break
		}
		time.Sleep(10 * time.Millisecond)
	}
	if len(bridge.got) != 1 {
		t.Fatalf("expected command forwarded to bridge, got %d", len(bridge.got))
	}
	if bridge.got[0].Type != protocol.CmdSlew {
		t.Errorf("unexpected forwarded type %q", bridge.got[0].Type)
	}

	// Gateway broadcasts a telemetry envelope; client should receive it.
	hub.Broadcast(protocol.Envelope{Type: protocol.TelemetryMount, NodeID: "n1", Payload: []byte(`{"ra":1}`)})
	conn.SetReadDeadline(time.Now().Add(2 * time.Second))
	var got protocol.Envelope
	if err := conn.ReadJSON(&got); err != nil {
		t.Fatalf("read telemetry: %v", err)
	}
	if got.Type != protocol.TelemetryMount {
		t.Errorf("expected telemetry.mount, got %q", got.Type)
	}
}
