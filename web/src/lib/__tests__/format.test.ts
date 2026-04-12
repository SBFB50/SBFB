// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 6 Phase D — Vitest unit tests for pure format helpers.
 *
 * Goal: exercise every branch of format.ts including the null /
 * undefined fall-throughs so a regression in the R-5 guarantee
 * ("render path never throws on missing fields") fails loudly.
 *
 * Uses vi.setSystemTime to freeze Date.now so relative-time
 * assertions stay deterministic.
 */

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  formatHash,
  formatMemoryMb,
  formatRelativeTime,
  formatUptime,
} from "../format";

describe("formatHash", () => {
  it("returns em-dash on null", () => {
    expect(formatHash(null)).toBe("—");
  });

  it("returns em-dash on undefined", () => {
    expect(formatHash(undefined)).toBe("—");
  });

  it("returns em-dash on empty string", () => {
    expect(formatHash("")).toBe("—");
  });

  it("returns the input unchanged when shorter than chars", () => {
    expect(formatHash("short")).toBe("short");
    expect(formatHash("short", 10)).toBe("short");
  });

  it("truncates and appends ellipsis when longer than chars", () => {
    expect(formatHash("abcdef1234567890deadbeef", 12)).toBe("abcdef123456…");
  });

  it("respects a custom char length", () => {
    expect(formatHash("abcdef", 3)).toBe("abc…");
  });

  it("handles boundary equal to chars (no truncation)", () => {
    expect(formatHash("abcdef", 6)).toBe("abcdef");
  });
});

describe("formatUptime", () => {
  it("returns 0s on zero", () => {
    expect(formatUptime(0)).toBe("0s");
  });

  it("returns 0s on negative", () => {
    expect(formatUptime(-42)).toBe("0s");
  });

  it("formats seconds only under a minute", () => {
    expect(formatUptime(42)).toBe("42s");
  });

  it("formats minutes + seconds", () => {
    expect(formatUptime(65)).toBe("1m 5s");
  });

  it("formats hours + minutes + seconds", () => {
    expect(formatUptime(3600 + 120 + 3)).toBe("1h 2m 3s");
  });

  it("always shows the minute segment when hours are present, even if zero", () => {
    expect(formatUptime(3601)).toBe("1h 0m 1s");
  });

  it("floors fractional input", () => {
    expect(formatUptime(59.999)).toBe("59s");
  });
});

describe("formatRelativeTime", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-04-11T12:00:00Z"));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("returns em-dash on null", () => {
    expect(formatRelativeTime(null)).toBe("—");
  });

  it("returns em-dash on undefined", () => {
    expect(formatRelativeTime(undefined)).toBe("—");
  });

  it("returns em-dash on invalid ISO string", () => {
    expect(formatRelativeTime("not-a-date")).toBe("—");
  });

  it("returns 'à l'instant' for a delta under 2 seconds", () => {
    expect(formatRelativeTime("2026-04-11T11:59:59Z")).toBe("à l'instant");
  });

  it("formats seconds ago", () => {
    expect(formatRelativeTime("2026-04-11T11:59:55Z")).toBe("il y a 5 s");
  });

  it("formats minutes ago", () => {
    expect(formatRelativeTime("2026-04-11T11:55:00Z")).toBe("il y a 5 min");
  });

  it("formats hours ago", () => {
    expect(formatRelativeTime("2026-04-11T09:00:00Z")).toBe("il y a 3 h");
  });

  it("formats days ago", () => {
    expect(formatRelativeTime("2026-04-09T12:00:00Z")).toBe("il y a 2 j");
  });

  it("formats a future timestamp as 'dans'", () => {
    expect(formatRelativeTime("2026-04-11T12:05:00Z")).toBe("dans 5 min");
  });

  it("accepts unix seconds as number", () => {
    const now = Date.now() / 1000;
    expect(formatRelativeTime(now - 10)).toBe("il y a 10 s");
  });

  it("handles the 'dans un instant' near-zero future branch", () => {
    expect(formatRelativeTime("2026-04-11T12:00:00.500Z")).toBe(
      "dans un instant",
    );
  });
});

describe("formatMemoryMb", () => {
  it("returns em-dash on null", () => {
    expect(formatMemoryMb(null)).toBe("—");
  });

  it("returns em-dash on undefined", () => {
    expect(formatMemoryMb(undefined)).toBe("—");
  });

  it("formats small values as MiB", () => {
    expect(formatMemoryMb(42)).toBe("42 MiB");
  });

  it("rounds fractional MiB values", () => {
    expect(formatMemoryMb(42.4)).toBe("42 MiB");
    expect(formatMemoryMb(42.6)).toBe("43 MiB");
  });

  it("switches to GiB at 1024 boundary", () => {
    expect(formatMemoryMb(1024)).toBe("1.0 GiB");
  });

  it("formats multi-gigabyte values with one decimal", () => {
    expect(formatMemoryMb(2048)).toBe("2.0 GiB");
    expect(formatMemoryMb(3072)).toBe("3.0 GiB");
  });

  it("handles zero", () => {
    expect(formatMemoryMb(0)).toBe("0 MiB");
  });
});
