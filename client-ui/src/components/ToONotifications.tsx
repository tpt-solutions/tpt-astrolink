// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

"use client";

import { useState } from "react";
import type { TooAlert } from "../lib/types";

/** ToONotifications shows Target-of-Opportunity transient alerts. */
export function ToONotifications({ alerts }: { alerts: TooAlert[] }) {
  const [dismissed, setDismissed] = useState<Set<string>>(new Set());

  const visible = alerts.filter((a) => !dismissed.has(a.objectId));

  if (visible.length === 0) return null;

  return (
    <div role="alert" style={{ position: "fixed", top: 16, right: 16, zIndex: 50, maxWidth: 320 }}>
      {visible.map((a) => (
        <div
          key={a.objectId}
          style={{
            background: "#2a1a00",
            border: "1px solid #ffd166",
            borderRadius: 8,
            padding: 12,
            marginBottom: 8,
          }}
        >
          <strong>Target of Opportunity</strong>
          <div>RA {a.ra.toFixed(3)}° · Dec {a.dec.toFixed(3)}°</div>
          <div>Δmag {a.magDelta.toFixed(2)} · conf {(a.confidence * 100).toFixed(0)}%</div>
          <button onClick={() => setDismissed((s) => new Set(s).add(a.objectId))}>Dismiss</button>
        </div>
      ))}
    </div>
  );
}
