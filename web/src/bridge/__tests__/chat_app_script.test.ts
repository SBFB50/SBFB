// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Chat v1 Phase F — execute the SHIPPED `examples/sbfb-chat` page in jsdom.
 *
 * Reads the exact `index.html` + `app.js` the archive carries and drives them
 * through a fake `SBFBBridge` (the host and the daemon are covered by their
 * own suites). What is pinned here is the app's own behaviour: which bridge
 * verb each control calls, that untrusted text is rendered as text, and that
 * a refused consent is explained to the user.
 */
import { readFileSync } from "node:fs";
import { resolve as pathResolve } from "node:path";

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const APP = pathResolve(process.cwd(), "../examples/sbfb-chat");
const HTML = readFileSync(pathResolve(APP, "index.html"), "utf8");
const APP_SRC = readFileSync(pathResolve(APP, "app.js"), "utf8");
const MANIFEST = JSON.parse(readFileSync(pathResolve(APP, "SBFB.json"), "utf8")) as {
  bridge: { methods: string[] };
};

type Call = { method: string; args: unknown[] };

interface FakeState {
  name: string;
  convs: Array<{
    id: string;
    title: string;
    is_admin: boolean;
    removed: boolean;
    members: Array<{ id: string; name: string; fingerprint: string; admin: boolean }>;
  }>;
  messages: Record<string, unknown[]>;
  mailbox: { enabled: boolean; servers: string[]; conversations: unknown[] };
  failIdentity?: string;
}

function installFakeBridge(state: FakeState, calls: Call[]) {
  let chatHandler: ((ev: unknown) => void) | null = null;
  const rec = (method: string, args: unknown[], value: unknown) => {
    calls.push({ method, args });
    return Promise.resolve(value);
  };
  class FakeBridge {
    chatIdentity() {
      calls.push({ method: "chatIdentity", args: [] });
      if (state.failIdentity) return Promise.reject(new Error(state.failIdentity));
      return Promise.resolve({ id: "me", name: state.name, fingerprint: "1111 2222" });
    }
    chatSetProfile(name: string) {
      state.name = name;
      return rec("chatSetProfile", [name], { ok: true });
    }
    chatList() {
      return rec("chatList", [], state.convs);
    }
    chatCreate(title: string) {
      const id = `c${state.convs.length + 1}`;
      state.convs.push({
        id,
        title,
        is_admin: true,
        removed: false,
        members: [{ id: "me", name: state.name, fingerprint: "1111 2222", admin: true }],
      });
      state.messages[id] = [];
      return rec("chatCreate", [title], { id });
    }
    chatMessages(id: string) {
      return rec("chatMessages", [id], state.messages[id] ?? []);
    }
    chatSend(id: string, text: string) {
      return rec("chatSend", [id, text], { ok: true });
    }
    chatInvite(id: string) {
      return rec("chatInvite", [id], { invite: "sbfb-chat:abc", expires_at: 1_900_000_000 });
    }
    chatAccept(link: string) {
      return rec("chatAccept", [link], { id: "c9" });
    }
    chatRemove(id: string, member: string) {
      return rec("chatRemove", [id, member], { ok: true });
    }
    chatMailbox(enabled?: boolean) {
      if (enabled !== undefined) state.mailbox.enabled = enabled;
      return rec("chatMailbox", enabled === undefined ? [] : [enabled], { ...state.mailbox });
    }
    onChat(cb: (ev: unknown) => void) {
      chatHandler = cb;
      return () => {};
    }
  }
  (globalThis as unknown as { SBFBBridge: unknown }).SBFBBridge = FakeBridge;
  return { push: (ev: unknown) => chatHandler?.(ev) };
}

function mountPage() {
  const body = HTML.slice(HTML.indexOf("<body>") + 6, HTML.indexOf("</body>"))
    .replace(/<script[\s\S]*?<\/script>/g, "");
  document.body.innerHTML = body;
}

function run() {
  (0, eval)(APP_SRC);
}

const $ = (id: string) => document.getElementById(id) as HTMLElement;
const click = (id: string) => ($(id) as HTMLButtonElement).click();
const type = (id: string, value: string) => {
  ($(id) as HTMLInputElement).value = value;
};

function freshState(over: Partial<FakeState> = {}): FakeState {
  return {
    name: "Alice",
    convs: [],
    messages: {},
    mailbox: { enabled: false, servers: ["anchor"], conversations: [] },
    ...over,
  };
}

describe("examples/sbfb-chat app.js (executed in jsdom)", () => {
  let calls: Call[];

  beforeEach(() => {
    calls = [];
    vi.useFakeTimers({ toFake: ["setInterval"] });
    mountPage();
  });

  afterEach(() => {
    vi.useRealTimers();
    document.body.innerHTML = "";
    delete (globalThis as unknown as { SBFBBridge?: unknown }).SBFBBridge;
  });

  it("declares exactly the chat verbs its SDK calls reach", () => {
    const used = new Set(
      [...APP_SRC.matchAll(/\.(chat[A-Z][A-Za-z]+)\(/g)].map((m) =>
        m[1].replace(/[A-Z]/g, (c) => `_${c.toLowerCase()}`),
      ),
    );
    expect(used.size).toBeGreaterThan(5);
    for (const verb of used) expect(MANIFEST.bridge.methods).toContain(verb);
  });

  it("first launch asks for a name, then shows the conversations", async () => {
    const state = freshState({ name: "" });
    installFakeBridge(state, calls);
    run();
    await vi.waitFor(() => expect($("onboarding").hidden).toBe(false));
    expect($("layout").hidden).toBe(true);
    type("profile-name", "  Alice  ");
    click("profile-save");
    await vi.waitFor(() => expect($("layout").hidden).toBe(false));
    expect(calls.find((c) => c.method === "chatSetProfile")?.args).toEqual(["Alice"]);
    expect($("identity-label").textContent).toContain("Alice");
    expect($("identity-label").textContent).toContain("1111 2222");
  });

  it("creates, opens, sends, and renders live messages as TEXT", async () => {
    const state = freshState();
    const bridge = installFakeBridge(state, calls);
    run();
    await vi.waitFor(() => expect($("layout").hidden).toBe(false));
    type("new-conv-title", "Projet");
    click("new-conv-btn");
    await vi.waitFor(() => expect($("chat-inner").hidden).toBe(false));
    expect($("chat-title").textContent).toBe("Projet");
    expect($("conv-list").textContent).toContain("Projet");

    type("composer-text", "bonjour");
    click("send-btn");
    await vi.waitFor(() =>
      expect(calls.find((c) => c.method === "chatSend")?.args).toEqual(["c1", "bonjour"]),
    );

    bridge.push({
      type: "message",
      conversation: "c1",
      from: "bob",
      from_name: "Bob",
      body: '<img src=x onerror="window.pwned=1">',
      sent_at: 1_800_000_000,
      mine: false,
    });
    const items = $("messages").querySelectorAll("li.message");
    expect(items).toHaveLength(1);
    expect(items[0].textContent).toContain("Bob");
    expect(items[0].textContent).toContain('<img src=x onerror="window.pwned=1">');
    expect($("messages").querySelector("img")).toBeNull();
  });

  it("Enter sends, Shift+Enter does not", async () => {
    const state = freshState();
    installFakeBridge(state, calls);
    run();
    await vi.waitFor(() => expect($("layout").hidden).toBe(false));
    type("new-conv-title", "X");
    click("new-conv-btn");
    await vi.waitFor(() => expect($("chat-inner").hidden).toBe(false));
    type("composer-text", "ligne");
    $("composer-text").dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", shiftKey: true }));
    expect(calls.some((c) => c.method === "chatSend")).toBe(false);
    $("composer-text").dispatchEvent(new KeyboardEvent("keydown", { key: "Enter" }));
    await vi.waitFor(() => expect(calls.some((c) => c.method === "chatSend")).toBe(true));
  });

  it("the mailbox stays off until the user turns it on", async () => {
    const state = freshState();
    installFakeBridge(state, calls);
    run();
    await vi.waitFor(() => expect($("mailbox-state").textContent).toContain("Désactivée"));
    expect(calls.filter((c) => c.method === "chatMailbox").every((c) => c.args.length === 0)).toBe(
      true,
    );
    click("mailbox-btn");
    await vi.waitFor(() => expect($("mailbox-state").textContent).toContain("Activée"));
    expect(calls.find((c) => c.args[0] === true)?.method).toBe("chatMailbox");
    expect($("mailbox-btn").getAttribute("aria-pressed")).toBe("true");
  });

  it("without a configured server the mailbox cannot be turned on", async () => {
    const state = freshState({
      mailbox: { enabled: false, servers: [], conversations: [] },
    });
    installFakeBridge(state, calls);
    run();
    await vi.waitFor(() =>
      expect($("mailbox-state").textContent).toContain("Aucun nœud boîte aux lettres"),
    );
    expect(($("mailbox-btn") as HTMLButtonElement).disabled).toBe(true);
  });

  it("a refused consent is explained, not shown raw", async () => {
    const state = freshState({ failIdentity: "method chat_identity not authorized" });
    installFakeBridge(state, calls);
    run();
    await vi.waitFor(() => expect($("notice").textContent).toContain("n'est pas autorisée"));
    expect($("notice").className).toContain("error");
    expect($("identity-label").textContent).toBe("Indisponible");
  });

  it("joining refuses anything that is not an invitation link", async () => {
    const state = freshState();
    installFakeBridge(state, calls);
    run();
    await vi.waitFor(() => expect($("layout").hidden).toBe(false));
    type("join-link", "https://example.com");
    click("join-btn");
    expect($("notice").textContent).toContain("n'est pas un lien");
    expect(calls.some((c) => c.method === "chatAccept")).toBe(false);
    type("join-link", "sbfb-chat:abc");
    click("join-btn");
    await vi.waitFor(() =>
      expect(calls.find((c) => c.method === "chatAccept")?.args).toEqual(["sbfb-chat:abc"]),
    );
  });
});
