// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

import { describe, expect, it } from "vitest";
import { formatTz } from "./timezones";

describe("formatTz", () => {
  const d = new Date("2026-07-20T02:15:00Z");

  it("formats a valid IANA timezone", () => {
    const out = formatTz(d, "Pacific/Auckland");
    expect(out).not.toBeNull();
    expect(out).toContain("2026");
  });

  it("returns null for missing timezone", () => {
    expect(formatTz(d, undefined)).toBeNull();
  });

  it("returns null for an invalid timezone", () => {
    expect(formatTz(d, "Not/AZone")).toBeNull();
  });
});
