// SPDX-License-Identifier: AGPL-3.0-or-later
/** Chat v1 (ADR_CHAT_V1) — host-side helpers: consent store + SSE parser. */
import { beforeEach, describe, expect, it } from "vitest";

import { getChatConsent, setChatConsent } from "../chat-consent";
import { parseSse } from "../useChatStream";

describe("chat consent store", () => {
  beforeEach(() => window.localStorage.clear());

  it("is unset until decided, then pinned to the exact archive", () => {
    expect(getChatConsent("hashA")).toBeNull();
    setChatConsent("hashA", "granted");
    expect(getChatConsent("hashA")).toBe("granted");
    // A new version of the app is a new archive: asked again.
    expect(getChatConsent("hashB")).toBeNull();
    setChatConsent("hashA", "denied");
    expect(getChatConsent("hashA")).toBe("denied");
  });

  it("fails closed on corrupted storage", () => {
    window.localStorage.setItem("sbfb.chat-consent.v1", "{not json");
    expect(getChatConsent("hashA")).toBeNull();
    window.localStorage.setItem("sbfb.chat-consent.v1", JSON.stringify({ hashA: "yes" }));
    expect(getChatConsent("hashA")).toBeNull();
  });
});

describe("parseSse", () => {
  it("extracts complete data frames and keeps the remainder", () => {
    const { data, rest } = parseSse('data: {"a":1}\n\n: keep-alive\n\ndata: {"b":2}\n\ndata: {"c"');
    expect(data).toEqual(['{"a":1}', '{"b":2}']);
    expect(rest).toBe('data: {"c"');
  });

  it("handles CRLF and multi-line data", () => {
    const { data, rest } = parseSse("data: line1\r\ndata: line2\r\n\r\n");
    expect(data).toEqual(["line1\nline2"]);
    expect(rest).toBe("");
  });
});
