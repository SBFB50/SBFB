/**
 * Sprint 6 audit fix F-1 — regression guard for the Ctrl+K keybind.
 *
 * The initial Phase C implementation matched `e.key === "k"`, which
 * silently failed when caps lock was on (`e.key === "K"`) and could
 * fail on non-QWERTY layouts where the "k" character lives on a
 * different physical key. The fix compares `e.code === "KeyK"`, a
 * layout- and case-independent physical key identifier.
 *
 * These tests dispatch synthetic KeyboardEvents with the lowercase,
 * uppercase, and physical-code shapes and assert the hook toggles
 * accordingly.
 */

import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { act, renderHook } from "@testing-library/react";

import { useCommandPalette } from "../useCommandPalette";

function dispatchCtrlK(init: KeyboardEventInit) {
  act(() => {
    document.dispatchEvent(new KeyboardEvent("keydown", init));
  });
}

describe("useCommandPalette", () => {
  beforeEach(() => {
    // Ensure no leftover listeners between cases.
    document.body.innerHTML = "";
  });

  afterEach(() => {
    document.body.innerHTML = "";
  });

  it("starts closed", () => {
    const { result } = renderHook(() => useCommandPalette());
    expect(result.current.open).toBe(false);
  });

  it("opens on Ctrl+K with lowercase 'k'", () => {
    const { result } = renderHook(() => useCommandPalette());
    dispatchCtrlK({ key: "k", code: "KeyK", ctrlKey: true });
    expect(result.current.open).toBe(true);
  });

  it("opens on Ctrl+K when caps lock flips e.key to 'K' (regression F-1)", () => {
    const { result } = renderHook(() => useCommandPalette());
    // Simulate caps lock: e.key is uppercase, but e.code stays KeyK.
    dispatchCtrlK({ key: "K", code: "KeyK", ctrlKey: true });
    expect(result.current.open).toBe(true);
  });

  it("opens on Cmd+K (macOS) via metaKey", () => {
    const { result } = renderHook(() => useCommandPalette());
    dispatchCtrlK({ key: "k", code: "KeyK", metaKey: true });
    expect(result.current.open).toBe(true);
  });

  it("opens when e.key is a non-Latin character but e.code is KeyK (non-QWERTY layouts)", () => {
    const { result } = renderHook(() => useCommandPalette());
    // Russian ЙЦУКЕН layout example: the physical K key produces "л".
    dispatchCtrlK({ key: "л", code: "KeyK", ctrlKey: true });
    expect(result.current.open).toBe(true);
  });

  it("ignores a bare K press with no modifier", () => {
    const { result } = renderHook(() => useCommandPalette());
    dispatchCtrlK({ key: "k", code: "KeyK" });
    expect(result.current.open).toBe(false);
  });

  it("ignores Ctrl + another key", () => {
    const { result } = renderHook(() => useCommandPalette());
    dispatchCtrlK({ key: "j", code: "KeyJ", ctrlKey: true });
    expect(result.current.open).toBe(false);
  });

  it("toggles: second Ctrl+K closes the palette", () => {
    const { result } = renderHook(() => useCommandPalette());
    dispatchCtrlK({ key: "k", code: "KeyK", ctrlKey: true });
    expect(result.current.open).toBe(true);
    dispatchCtrlK({ key: "k", code: "KeyK", ctrlKey: true });
    expect(result.current.open).toBe(false);
  });

  it("toggle() imperative helper flips state", () => {
    const { result } = renderHook(() => useCommandPalette());
    act(() => result.current.toggle());
    expect(result.current.open).toBe(true);
    act(() => result.current.toggle());
    expect(result.current.open).toBe(false);
  });

  it("setOpen(false) closes after a Ctrl+K open", () => {
    const { result } = renderHook(() => useCommandPalette());
    dispatchCtrlK({ key: "k", code: "KeyK", ctrlKey: true });
    act(() => result.current.setOpen(false));
    expect(result.current.open).toBe(false);
  });
});
