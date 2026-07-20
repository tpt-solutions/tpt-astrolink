// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

package mqttbridge

import (
	"testing"

	"github.com/tptsolutions/tpt-astrolink/cloud-core/internal/protocol"
)

func TestNodeIDFromTopic(t *testing.T) {
	cases := map[string]string{
		"tpt/v1/node-42/tele/mount": "node-42",
		"tpt/v1/abc/evt/too":        "abc",
		"tpt/v1/x/status":           "x",
	}
	for topic, want := range cases {
		if got := nodeIDFromTopic(topic); got != want {
			t.Errorf("nodeIDFromTopic(%q) = %q, want %q", topic, got, want)
		}
	}
}

func TestSendCommandTopic(t *testing.T) {
	var published string
	b := NewBridge(func(topic string, _ []byte) error {
		published = topic
		return nil
	}, nil)
	env := protocol.Envelope{Type: protocol.CmdSlew, ID: "1", NodeID: "node-9"}
	if err := b.SendCommand(env); err != nil {
		t.Fatal(err)
	}
	if published != "tpt/v1/node-9/cmd/all" {
		t.Errorf("unexpected command topic %q", published)
	}
}
