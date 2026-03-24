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

describe("GridView", () => {
  it("moves a pad via mouse drag gestures", () => {
    const onMovePad = vi.fn();

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

    fireEvent.mouseDown(source, { button: 0, buttons: 1 });
    fireEvent.mouseEnter(target, { buttons: 1 });
    fireEvent.mouseUp(target, { button: 0, buttons: 0 });

    expect(onMovePad).toHaveBeenCalledWith("r0c0", "r0c1");
  });

  it("clears pointer drag state when released off-grid", () => {
    const onMovePad = vi.fn();

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

    fireEvent.mouseDown(source, { button: 0, buttons: 1 });
    fireEvent.mouseUp(window, { button: 0, buttons: 0 });
    fireEvent.mouseEnter(target, { buttons: 0 });
    fireEvent.mouseUp(target, { button: 0, buttons: 0 });

    expect(onMovePad).not.toHaveBeenCalled();
  });

  it("disables native html drag handling so mouse release stays in app control", () => {
    render(
      <GridView
        pads={createPads()}
        selectedPadId="r0c0"
        onSelectPad={() => {}}
      />,
    );

    expect(
      screen.getByRole("button", { name: "Finder" }).getAttribute("draggable"),
    ).toBeNull();
  });
});
