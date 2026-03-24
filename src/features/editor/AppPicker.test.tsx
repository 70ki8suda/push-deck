// @vitest-environment jsdom

import { afterEach, beforeAll, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { AppPicker, getEffectiveAppPickerOptions } from "./AppPicker";
import type { AppPickerOption } from "../../lib/types";

const TERMINAL: AppPickerOption = {
  bundleId: "com.apple.Terminal",
  appName: "Terminal",
};

const OPTIONS: readonly AppPickerOption[] = [
  {
    bundleId: "com.apple.finder",
    appName: "Finder",
  },
  {
    bundleId: "com.apple.Safari",
    appName: "Safari",
  },
  TERMINAL,
];

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

describe("AppPicker", () => {
  it("keeps fallback app options available before runtime app discovery loads", () => {
    expect(getEffectiveAppPickerOptions([], null).map((app) => app.appName)).toEqual([
      "Finder",
      "Safari",
      "Google Chrome",
      "Terminal",
    ]);
  });

  it("filters app candidates from the search input and selects a result", async () => {
    const user = userEvent.setup();
    const onSelectApp = vi.fn();

    render(
      <AppPicker
        options={OPTIONS}
        selectedApp={null}
        onSelectApp={onSelectApp}
      />,
    );

    const input = screen.getByRole("combobox", { name: "App picker" });
    await user.click(input);
    await user.type(input, "term");

    expect(screen.getByRole("option", { name: /Terminal/ })).toBeTruthy();
    expect(screen.queryByRole("option", { name: /Finder/ })).toBeNull();

    await user.click(screen.getByRole("option", { name: /Terminal/ }));

    expect(onSelectApp).toHaveBeenCalledWith(TERMINAL);
  });

  it("clears the selected app from the combobox", async () => {
    const user = userEvent.setup();
    const onSelectApp = vi.fn();

    render(
      <AppPicker
        options={OPTIONS}
        selectedApp={TERMINAL}
        onSelectApp={onSelectApp}
      />,
    );

    expect(
      (screen.getByRole("combobox", { name: "App picker" }) as HTMLInputElement)
        .value,
    ).toBe("Terminal");

    await user.click(
      screen.getByRole("button", { name: "Clear app selection" }),
    );

    expect(onSelectApp).toHaveBeenCalledWith(null);
  });
});
