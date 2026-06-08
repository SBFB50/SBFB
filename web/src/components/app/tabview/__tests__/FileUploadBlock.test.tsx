// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 9 Phase E — FileUploadBlock Vitest tests.
 *
 * 8 tests covering the drop zone render, accept list display,
 * max size display, error state, result state, and drag events.
 */

import { render, screen, fireEvent } from "@testing-library/react";
import { afterEach, describe, it, expect, vi } from "vitest";
import { FileUploadBlock } from "../blocks/FileUploadBlock";
import type { TabBlockFileUpload } from "../schema";

const BASE_BLOCK: TabBlockFileUpload = {
  kind: "file_upload",
  label: "Deposer un fichier",
  accept: ["image/*", "application/pdf"],
  max_size_bytes: 50 * 1024 * 1024,
};

/** A File whose reported `size` is forced (jsdom keeps the byte length). */
function fileOf(name: string, bytes: number): File {
  const f = new File(["x"], name, { type: "application/pdf" });
  Object.defineProperty(f, "size", { value: bytes });
  return f;
}

function jsonResponse(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "content-type": "application/json" },
  });
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("FileUploadBlock", () => {
  it("renders the label", () => {
    render(<FileUploadBlock block={BASE_BLOCK} />);
    expect(screen.getByText("Deposer un fichier")).toBeTruthy();
  });

  it("renders the accept list", () => {
    render(<FileUploadBlock block={BASE_BLOCK} />);
    expect(
      screen.getByText(/image\/\*, application\/pdf/),
    ).toBeTruthy();
  });

  it("renders the max size in human-readable form", () => {
    render(<FileUploadBlock block={BASE_BLOCK} />);
    expect(screen.getByText(/50\.0 Mo/)).toBeTruthy();
  });

  it("renders a drop zone with data-testid", () => {
    render(<FileUploadBlock block={BASE_BLOCK} />);
    const dropzone = screen.getByTestId("file-upload-dropzone");
    expect(dropzone).toBeTruthy();
  });

  it("renders with a small max size in Ko", () => {
    const block: TabBlockFileUpload = {
      ...BASE_BLOCK,
      max_size_bytes: 512 * 1024,
    };
    render(<FileUploadBlock block={block} />);
    expect(screen.getByText(/512\.0 Ko/)).toBeTruthy();
  });

  it("renders with custom accept list", () => {
    const block: TabBlockFileUpload = {
      ...BASE_BLOCK,
      accept: ["image/png"],
    };
    render(<FileUploadBlock block={block} />);
    expect(screen.getByText(/image\/png/)).toBeTruthy();
  });

  it("applies drag-over styling on dragOver event", () => {
    render(<FileUploadBlock block={BASE_BLOCK} />);
    const dropzone = screen.getByTestId("file-upload-dropzone");
    fireEvent.dragOver(dropzone);
    expect(dropzone.className).toContain("border-primary");
  });

  it("removes drag styling on dragLeave event", () => {
    render(<FileUploadBlock block={BASE_BLOCK} />);
    const dropzone = screen.getByTestId("file-upload-dropzone");
    fireEvent.dragOver(dropzone);
    fireEvent.dragLeave(dropzone);
    expect(dropzone.className).not.toContain("bg-primary/5");
  });

  // --- T14 (Sprint 74 Phase G): cover the upload branches ---

  it("rejects a file larger than max_size_bytes WITHOUT fetching", () => {
    const fetchSpy = vi.fn();
    vi.stubGlobal("fetch", fetchSpy);
    const block: TabBlockFileUpload = { ...BASE_BLOCK, max_size_bytes: 1 };
    const { container } = render(<FileUploadBlock block={block} />);
    const input = container.querySelector(
      'input[type="file"]',
    ) as HTMLInputElement;
    fireEvent.change(input, { target: { files: [fileOf("big.pdf", 2)] } });
    expect(screen.getByTestId("upload-error").textContent).toMatch(
      /taille maximale/,
    );
    expect(fetchSpy).not.toHaveBeenCalled();
  });

  it("uploads a file and renders the result on a 200 response", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () =>
        jsonResponse(200, {
          sha256: "a".repeat(64),
          size: 123,
          original_name: "doc.pdf",
        }),
      ),
    );
    const { container } = render(<FileUploadBlock block={BASE_BLOCK} />);
    const input = container.querySelector(
      'input[type="file"]',
    ) as HTMLInputElement;
    fireEvent.change(input, { target: { files: [fileOf("doc.pdf", 123)] } });

    const result = await screen.findByTestId("upload-result");
    expect(result.textContent).toMatch(/doc\.pdf/);
    expect(result.textContent).toMatch(/aaaaaaaaaaaaaaaa/); // sha256 slice(0,16)
    expect(result.textContent).toMatch(/123 o/);
  });

  it("surfaces the server `detail` on a non-ok response", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => jsonResponse(413, { detail: "fichier rejete" })),
    );
    const { container } = render(<FileUploadBlock block={BASE_BLOCK} />);
    const input = container.querySelector(
      'input[type="file"]',
    ) as HTMLInputElement;
    fireEvent.change(input, { target: { files: [fileOf("doc.pdf", 10)] } });

    const err = await screen.findByTestId("upload-error");
    expect(err.textContent).toBe("fichier rejete");
  });

  it("falls back to a generic message on a non-ok response without detail", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => jsonResponse(500, {})),
    );
    const { container } = render(<FileUploadBlock block={BASE_BLOCK} />);
    const input = container.querySelector(
      'input[type="file"]',
    ) as HTMLInputElement;
    fireEvent.change(input, { target: { files: [fileOf("doc.pdf", 10)] } });

    const err = await screen.findByTestId("upload-error");
    expect(err.textContent).toMatch(/Erreur serveur \(500\)/);
  });

  it("surfaces a network error when fetch rejects", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => {
        throw new Error("network down");
      }),
    );
    const { container } = render(<FileUploadBlock block={BASE_BLOCK} />);
    const input = container.querySelector(
      'input[type="file"]',
    ) as HTMLInputElement;
    fireEvent.change(input, { target: { files: [fileOf("doc.pdf", 10)] } });

    const err = await screen.findByTestId("upload-error");
    expect(err.textContent).toBe("network down");
  });

  it("shows the uploading state while the request is in flight", async () => {
    let release: (() => void) | undefined;
    vi.stubGlobal(
      "fetch",
      vi.fn(
        () =>
          new Promise<Response>((resolve) => {
            release = () =>
              resolve(
                jsonResponse(200, {
                  sha256: "b".repeat(64),
                  size: 1,
                  original_name: "d.pdf",
                }),
              );
          }),
      ),
    );
    const { container } = render(<FileUploadBlock block={BASE_BLOCK} />);
    const input = container.querySelector(
      'input[type="file"]',
    ) as HTMLInputElement;
    fireEvent.change(input, { target: { files: [fileOf("d.pdf", 1)] } });
    expect(await screen.findByText(/Téléversement en cours/)).toBeTruthy();
    release?.();
    await screen.findByTestId("upload-result");
  });

  it("opens the file picker on Enter and Space keydown but not other keys", () => {
    const clickSpy = vi.spyOn(HTMLInputElement.prototype, "click");
    render(<FileUploadBlock block={BASE_BLOCK} />);
    const dropzone = screen.getByTestId("file-upload-dropzone");
    fireEvent.keyDown(dropzone, { key: "Enter" });
    fireEvent.keyDown(dropzone, { key: " " });
    const afterValidKeys = clickSpy.mock.calls.length;
    expect(afterValidKeys).toBeGreaterThanOrEqual(2); // both keys opened it
    fireEvent.keyDown(dropzone, { key: "a" }); // ignored — no new open
    expect(clickSpy.mock.calls.length).toBe(afterValidKeys);
    clickSpy.mockRestore();
  });

  it("opens the file picker on click", () => {
    const clickSpy = vi.spyOn(HTMLInputElement.prototype, "click");
    render(<FileUploadBlock block={BASE_BLOCK} />);
    fireEvent.click(screen.getByTestId("file-upload-dropzone"));
    expect(clickSpy).toHaveBeenCalled();
    clickSpy.mockRestore();
  });

  it("ignores a drop with no file and an input change with no file", () => {
    const fetchSpy = vi.fn();
    vi.stubGlobal("fetch", fetchSpy);
    const { container } = render(<FileUploadBlock block={BASE_BLOCK} />);
    const dropzone = screen.getByTestId("file-upload-dropzone");
    fireEvent.drop(dropzone, { dataTransfer: { files: [] } });
    const input = container.querySelector(
      'input[type="file"]',
    ) as HTMLInputElement;
    fireEvent.change(input, { target: { files: [] } });
    expect(fetchSpy).not.toHaveBeenCalled();
    expect(screen.queryByTestId("upload-error")).toBeNull();
  });

  it("uploads via drag-and-drop drop event", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () =>
        jsonResponse(200, {
          sha256: "c".repeat(64),
          size: 5,
          original_name: "dropped.pdf",
        }),
      ),
    );
    render(<FileUploadBlock block={BASE_BLOCK} />);
    const dropzone = screen.getByTestId("file-upload-dropzone");
    fireEvent.drop(dropzone, {
      dataTransfer: { files: [fileOf("dropped.pdf", 5)] },
    });
    expect((await screen.findByTestId("upload-result")).textContent).toMatch(
      /dropped\.pdf/,
    );
  });
});
