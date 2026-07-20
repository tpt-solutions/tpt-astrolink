// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

package main

import (
	"encoding/json"
	"fmt"
	"time"

	"github.com/tptsolutions/tpt-astrolink/cloud-core/internal/protocol"
)

func jsonUnmarshal(data []byte, v any) error { return json.Unmarshal(data, v) }

func parseTS(s string) (time.Time, bool) {
	t, err := time.Parse(time.RFC3339Nano, s)
	if err != nil {
		return time.Time{}, false
	}
	return t, true
}

func cmdPayload() json.RawMessage {
	b, _ := json.Marshal(protocol.Coords{RA: 12.5, Dec: -30, Epoch: "J2000"})
	return b
}

func uuidLike(conn int, seq int64) string {
	return fmt.Sprintf("c%d-%d", conn, seq)
}
