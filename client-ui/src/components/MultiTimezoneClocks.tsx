// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

"use client";

import { useEffect, useState } from "react";
import { formatTz } from "../lib/timezones";

const NODE_TZ = process.env.NEXT_PUBLIC_NODE_TZ; // e.g. "Pacific/Auckland"
const RESEARCHER_TZ = process.env.NEXT_PUBLIC_RESEARCHER_TZ; // e.g. "America/Los_Angeles"

function Clock({ label, tz }: { label: string; tz?: string }) {
  const [now, setNow] = useState<string | null>(null);
  useEffect(() => {
    const tick = () => setNow(formatTz(new Date(), tz));
    tick();
    const id = setInterval(tick, 1000);
    return () => clearInterval(id);
  }, [tz]);
  if (!tz || !now) return null;
  return (
    <span style={{ marginLeft: 12 }}>
      {label}: <strong>{now}</strong>
    </span>
  );
}

/** MultiTimezoneClocks shows the observer's local time plus any configured
 *  node/researcher timezones, for distributed multi-timezone monitoring. */
export function MultiTimezoneClocks() {
  const local = Intl.DateTimeFormat().resolvedOptions().timeZone;
  return (
    <div style={{ fontSize: 13 }}>
      <Clock label="Local" tz={local} />
      <Clock label="Node" tz={NODE_TZ} />
      <Clock label="Researcher" tz={RESEARCHER_TZ} />
    </div>
  );
}
