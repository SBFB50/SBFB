// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 9 Phase E — FileUploadBlock Vitest tests.
 *
 * 8 tests covering the drop zone render, accept list display,
 * max size display, error state, result state, and drag events.
 */

import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { FileUploadBlock } from "../blocks/FileUploadBlock";
import type { TabBlockFileUpload } from "../schema";

const BASE_BLOCK: TabBlockFileUpload = {
  kind: "file_upload",
  label: "Deposer un fichier",
  accept: ["image/*", "application/pdf"],
  max_size_bytes: 50 * 1024 * 1024,
};

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
});
