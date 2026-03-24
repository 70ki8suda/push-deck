// @vitest-environment jsdom

import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { GridView } from "./GridView";
import type { PadBinding } from "../../lib/types";

afterEach(() => {
  cleanup();
});

function createPads(): PadBinding[] {
  return Array.from({ length: 64 }, (_, index) => ({
    padId: `r${Math.floor(index / 8)}c${index % 8}`,
    label: index === 0 ? "Finder" : index === 1 ? "Terminal" : "",
    color: index === 0 ? "green" : index === 1 ? "blue" : "off",
    action:
      index === 0
        ? {
            type: "launch_or_focus_app",
            bundleId: "com.apple.finder",
            appName: "Finder",
          }
        : index === 1
          ? {
              type: "launch_or_focus_app",
              bundleId: "com.apple.Terminal",
              appName: "Terminal",
            }
          : {
              type: "unassigned",
            },
  }));
}

function createDataTransfer() {
  const data = new Map<string, string>();

  return {
    effectAllowed: "",
    dropEffect: "",
    setData(type: string, value: string) {
      data.set(type, value);
    },
    getData(type: string) {
      return data.get(type) ?? "";
    },
  };
}

describe("GridView", () => {
  it("moves a pad when drag data is available on drop", () => {
    const onMovePad = vi.fn();
    const dataTransfer = createDataTransfer();

    render(
      <GridView
        pads={createPads()}
        selectedPadId="r0c0"
        onSelectPad={() => {}}
        onMovePad={onMovePad}
      />,
    );

    const source = screen.getByRole("button", { name: "Finder" });
    const target = screen.getByRole("button", { name: "Terminal" });

    fireEvent.dragStart(source, { dataTransfer });
    fireEvent.dragOver(target, { dataTransfer });
    fireEvent.drop(target, { dataTransfer });

    expect(onMovePad).toHaveBeenCalledWith("r0c0", "r0c1");
  });

  it("still moves a pad when the drop event payload is empty", () => {
    const onMovePad = vi.fn();
    const dragStartDataTransfer = createDataTransfer();
    const dropDataTransfer = createDataTransfer();

    render(
      <GridView
        pads={createPads()}
        selectedPadId="r0c0"
        onSelectPad={() => {}}
        onMovePad={onMovePad}
      />,
    );

    const source = screen.getByRole("button", { name: "Finder" });
    const target = screen.getByRole("button", { name: "Terminal" });

    fireEvent.dragStart(source, { dataTransfer: dragStartDataTransfer });
    fireEvent.dragOver(target, { dataTransfer: dropDataTransfer });
    fireEvent.drop(target, { dataTransfer: dropDataTransfer });

    expect(onMovePad).toHaveBeenCalledWith("r0c0", "r0c1");
  });
});
