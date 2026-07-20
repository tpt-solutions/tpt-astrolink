// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

// Pure helpers for multi-timezone formatting (unit-testable).

/** formatTz formats a Date in the given IANA timezone. Returns null for an
 *  invalid timezone or when none is provided. */
export function formatTz(date: Date, tz?: string): string | null {
  if (!tz) return null;
  try {
    return new Intl.DateTimeFormat("en-GB", {
      timeZone: tz,
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
      year: "numeric",
      month: "short",
      day: "2-digit",
    }).format(date);
  } catch {
    return null;
  }
}
