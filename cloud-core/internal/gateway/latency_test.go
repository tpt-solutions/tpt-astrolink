// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

package gateway_test

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"sort"
	"strings"
	"testing"
	"time"

	"github.com/gorilla/websocket"
	"github.com/tptsolutions/tpt-astrolink/cloud-core/internal/gateway"
	"github.com/tptsolutions/tpt-astrolink/cloud-core/internal/protocol"
)

// loopbackBridge simulates the full Cloud Core -> MQTT -> Edge Agent ->
// MQTT -> Cloud Core round trip. On receiving a command it immediately
// echoes an ack back through the hub, exactly as the real MQTT bridge
// does when the edge agent acknowledges a slew. This lets the latency
// test exercise the real WebSocket routing + broadcast path rather than
// poking the hub directly.
type loopbackBridge struct {
	hub *gateway.Hub
}

func (b *loopbackBridge) SendCommand(env protocol.Envelope) error {
	b.hub.Broadcast(protocol.Envelope{
		Type:   protocol.MsgAck,
		ID:     env.ID,
		NodeID: env.NodeID,
		TS:     time.Now().UTC().Format(time.RFC3339Nano),
	})
	return nil
}

func newLatencyServer(tb testing.TB) (*httptest.Server, *gateway.Hub) {
	bridge := &loopbackBridge{}
	hub := gateway.NewHub(bridge)
	bridge.hub = hub
	go hub.Run()
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		hub.ServeWS(w, r)
	}))
	tb.Cleanup(srv.Close)
	return srv, hub
}

// TestCommandLatencyE2E validates the sub-second command target
// (Phase 6) end to end: client command -> gateway read loop -> bridge
// -> hub broadcast -> client ack. It asserts the p50 round-trip stays
// far under 1 second and p99 stays bounded.
func TestCommandLatencyE2E(t *testing.T) {
	srv, _ := newLatencyServer(t)
	c := dial(t, "ws"+strings.TrimPrefix(srv.URL, "http"))

	cmd := protocol.Envelope{
		Type:   protocol.CmdSlew,
		ID:     "lat-1",
		NodeID: "node-A",
		Payload: mustJSON(t, protocol.Coords{RA: 12.5, Dec: -30, Epoch: "J2000"}),
	}
	body, _ := json.Marshal(cmd)

	const n = 200
	lat := make([]time.Duration, 0, n)
	for i := 0; i < n; i++ {
		start := time.Now()
		if err := c.WriteMessage(websocket.TextMessage, body); err != nil {
			t.Fatalf("write: %v", err)
		}
		if got := readAck(t, c, "lat-1"); got.Type != protocol.MsgAck {
			t.Fatalf("unexpected message %q", got.Type)
		}
		lat = append(lat, time.Since(start))
	}

	p50 := percentile(lat, 50)
	p95 := percentile(lat, 95)
	p99 := percentile(lat, 99)
	t.Logf("command latency e2e: p50=%s p95=%s p99=%s (n=%d)", p50, p95, p99, n)

	if p50 >= time.Second {
		t.Fatalf("p50 latency %s exceeds sub-second target", p50)
	}
	if p99 >= 5*time.Second {
		t.Fatalf("p99 latency %s too high (target < 5s)", p99)
	}
}

func readAck(t *testing.T, c *websocket.Conn, id string) protocol.Envelope {
	t.Helper()
	c.SetReadDeadline(time.Now().Add(2 * time.Second))
	for {
		_, data, err := c.ReadMessage()
		if err != nil {
			t.Fatalf("read ack: %v", err)
		}
		var env protocol.Envelope
		if err := json.Unmarshal(data, &env); err != nil {
			continue
		}
		if env.Type == protocol.MsgAck && env.ID == id {
			return env
		}
	}
}

func percentile(lat []time.Duration, p int) time.Duration {
	sort.Slice(lat, func(i, j int) bool { return lat[i] < lat[j] })
	if len(lat) == 0 {
		return 0
	}
	idx := (p * len(lat)) / 100
	if idx >= len(lat) {
		idx = len(lat) - 1
	}
	return lat[idx]
}

// BenchmarkCommandLatencyE2E reports the steady-state command -> ack
// round-trip throughput/latency for regression tracking.
func BenchmarkCommandLatencyE2E(b *testing.B) {
	srv, _ := newLatencyServer(b)
	c, _, err := websocket.DefaultDialer.Dial("ws"+strings.TrimPrefix(srv.URL, "http"), nil)
	if err != nil {
		b.Fatalf("dial: %v", err)
	}
	defer c.Close()

	cmd := protocol.Envelope{
		Type:   protocol.CmdSlew,
		ID:     "bench",
		NodeID: "node-A",
		Payload: json.RawMessage(`{"ra":1,"dec":1,"epoch":"J2000"}`),
	}
	body, _ := json.Marshal(cmd)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		start := time.Now()
		if err := c.WriteMessage(websocket.TextMessage, body); err != nil {
			b.Fatalf("write: %v", err)
		}
		readAckBench(b, c, "bench")
		_ = time.Since(start)
	}
}

func readAckBench(b *testing.B, c *websocket.Conn, id string) {
	b.Helper()
	c.SetReadDeadline(time.Now().Add(2 * time.Second))
	for {
		_, data, err := c.ReadMessage()
		if err != nil {
			b.Fatalf("read ack: %v", err)
		}
		var env protocol.Envelope
		if err := json.Unmarshal(data, &env); err != nil {
			continue
		}
		if env.Type == protocol.MsgAck && env.ID == id {
			return
		}
	}
}
