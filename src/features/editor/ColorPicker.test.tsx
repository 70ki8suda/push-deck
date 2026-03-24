// @vitest-environment jsdom

import { afterEach, beforeAll, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ColorPicker } from "./ColorPicker";

beforeAll(() => {
  vi.stubGlobal(
    "ResizeObserver",
    class ResizeObserver {
      observe() {}

      unobserve() {}

      disconnect() {}
    },
  );

  Object.defineProperty(window.HTMLElement.prototype, "scrollIntoView", {
    configurable: true,
    value: vi.fn(),
  });
});

afterEach(() => {
  cleanup();
});

describe("ColorPicker", () => {
  it("shows the selected color in the input", () => {
    render(
      <ColorPicker
        selectedColor="blue"
        onSelectColor={() => {}}
      />,
    );

    expect(
      (screen.getByRole("combobox", { name: "Pad color" }) as HTMLInputElement).value,
    ).toBe("Blue");
  });

  it("filters color options from the search input and selects a result", async () => {
    const user = userEvent.setup();
    const onSelectColor = vi.fn();

    render(
      <ColorPicker
        selectedColor="off"
        onSelectColor={onSelectColor}
      />,
    );

    const input = screen.getByRole("combobox", { name: "Pad color" });
    await user.click(input);
    await user.clear(input);
    await user.type(input, "indi");

    expect(screen.getByRole("option", { name: /Indigo/ })).toBeTruthy();
    expect(screen.queryByRole("option", { name: /Green/ })).toBeNull();

    await user.click(screen.getByRole("option", { name: /Indigo/ }));

    expect(onSelectColor).toHaveBeenCalledWith("indigo");
  });
});
