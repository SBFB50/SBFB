// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 20 Phase B — PanicWipeKeybind unit tests.
 *
 * Covers two behaviours declared by the plan §5.3 :
 *
 *   14. `five_taps_within_3s_triggers_wipe` : the daemon-side
 *       `triggerPanicWipe` function is invoked exactly once.
 *   15. `four_taps_or_slow_does_not_trigger` : fewer than 5
 *       chord presses, or 5 presses spread across more than
 *       `WINDOW_MS`, must NOT invoke the trigger.
 *
 * The tests inject a mock `triggerImpl` so the component never
 * reaches real network, matching the hook contract documented
 * in `PanicWipeKeybind.tsx`.
 */

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { render, act } from "@testing-library/react";

import { PanicWipeKeybind, WINDOW_MS } from "../PanicWipeKeybind";

function fireChord() {
  const event = new KeyboardEvent("keydown", {
    key: "w",
    ctrlKey: true,
    shiftKey: true,
    altKey: true,
    metaKey: false,
    bubbles: true,
    cancelable: true,
  });
  window.dispatchEvent(event);
}

describe("<PanicWipeKeybind>", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("five_taps_within_3s_triggers_wipe", async () => {
    const trigger = vi.fn().mockResolvedValue({
      kind: "data",
      status: 200,
      body: { wiped: true as const },
    });
    render(
      <PanicWipeKeybind
        coordinatorBaseUrl="http://127.0.0.1:9999"
        triggerImpl={trigger}
      />,
    );

    // Fire 5 chords inside the window — each separated by a few
    // fake-timer ms so the sliding window still holds all of them.
    for (let i = 0; i < 5; i += 1) {
      act(() => {
        fireChord();
      });
      await act(async () => {
        vi.advanceTimersByTime(100);
      });
    }

    expect(trigger).toHaveBeenCalledTimes(1);
    expect(trigger).toHaveBeenCalledWith("http://127.0.0.1:9999");
  });

  it("four_taps_or_slow_does_not_trigger", async () => {
    const trigger = vi.fn();
    render(
      <PanicWipeKeybind
        coordinatorBaseUrl="http://127.0.0.1:9999"
        triggerImpl={trigger}
      />,
    );

    // 4 fast taps — below the threshold.
    for (let i = 0; i < 4; i += 1) {
      act(() => {
        fireChord();
      });
      await act(async () => {
        vi.advanceTimersByTime(50);
      });
    }
    expect(trigger).not.toHaveBeenCalled();

    // Advance well beyond WINDOW_MS so the sliding filter drops
    // every tap accumulated above. Without this, the 4 fast taps
    // combine with the first slow tap and the count reaches 5
    // inside the window, which is the OPPOSITE of what this
    // scenario is meant to exercise.
    await act(async () => {
      vi.advanceTimersByTime(WINDOW_MS * 2);
    });

    // 5 slow taps: spread beyond WINDOW_MS so the sliding
    // filter keeps only the most recent one each iteration and
    // the count never reaches 5.
    for (let i = 0; i < 5; i += 1) {
      act(() => {
        fireChord();
      });
      await act(async () => {
        vi.advanceTimersByTime(WINDOW_MS + 100);
      });
    }
    expect(trigger).not.toHaveBeenCalled();
  });
});
