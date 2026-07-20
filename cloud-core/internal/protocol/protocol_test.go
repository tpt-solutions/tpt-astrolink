// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

package protocol

import (
	"encoding/json"
	"strings"
	"testing"
)

func TestEnvelopeRoundTrip(t *testing.T) {
	env := Envelope{
		Type:   CmdSlew,
		ID:     "abc",
		NodeID: "node-1",
		Payload: mustJSON(t, Coords{RA: 12.5, Dec: -30, Epoch: "J2000"}),
	}
	b, err := json.Marshal(env)
	if err != nil {
		t.Fatal(err)
	}
	var out Envelope
	if err := json.Unmarshal(b, &out); err != nil {
		t.Fatal(err)
	}
	if out.Type != CmdSlew || out.NodeID != "node-1" {
		t.Fatalf("unexpected envelope: %+v", out)
	}
	var c Coords
	if err := json.Unmarshal(out.Payload, &c); err != nil {
		t.Fatal(err)
	}
	if c.RA != 12.5 || c.Dec != -30 {
		t.Fatalf("coords mismatch: %+v", c)
	}
}

func TestIsCommandTypes(t *testing.T) {
	for _, ty := range []string{CmdSlew, CmdSlewStop, CmdFocus, CmdFocusRel, CmdWeather, CmdImagingStart, CmdImagingStop, CmdSubscribe} {
		if !strings.HasPrefix(ty, "cmd.") {
			t.Errorf("unexpected command type %q", ty)
		}
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
