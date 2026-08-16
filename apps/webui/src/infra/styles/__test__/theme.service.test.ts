// @vitest-environment jsdom
import { ReflectiveInjector } from "injection-js";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { controlBrowser } from "@/__test__/support/browser";
import { DOCUMENT } from "@/infra/platform/injection";
import { provideStorages } from "@/infra/storage/context";
import { provideStyles } from "../context";
import { ThemeService } from "../theme.service";

const scopes: ThemeService[] = [];
function createTheme(target = document): ThemeService {
  const service = ReflectiveInjector.resolveAndCreate([
    { provide: DOCUMENT, useValue: target },
    ...provideStorages(),
    ...provideStyles(),
  ]).get(ThemeService);
  scopes.push(service);
  return service;
}

beforeEach(() => {
  localStorage.clear();
  document.documentElement.className = "";
});
afterEach(() => {
  for (const scope of scopes.splice(0)) scope.dispose();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

describe("theme scope", () => {
  it.each([
    [null, false, "system", "light"],
    [null, true, "system", "dark"],
    ["system", true, "system", "dark"],
    ["dark", false, "dark", "dark"],
    ["light", true, "light", "light"],
    ["invalid", true, "system", "dark"],
  ])(
    "initializes preference %s with system dark=%s",
    (stored, dark, preference, color) => {
      controlBrowser({ dark: Boolean(dark) });
      if (stored !== null)
        localStorage.setItem("prefers-color-scheme", String(stored));
      const theme = createTheme();
      theme.setup();
      expect(theme.preference.get()).toBe(preference);
      expect(theme.colorTheme.get()).toBe(color);
      expect(document.documentElement.classList.contains(String(color))).toBe(
        true,
      );
    },
  );

  it("updates same-page signals, DOM and persistence synchronously", () => {
    controlBrowser({ dark: true });
    const theme = createTheme();
    theme.setup();
    theme.setPreference("light");
    expect(theme.preference.get()).toBe("light");
    expect(theme.colorTheme.get()).toBe("light");
    expect(document.documentElement.classList.contains("light")).toBe(true);
    expect(localStorage.getItem("prefers-color-scheme")).toBe("light");
    theme.setPreference("system");
    expect(theme.colorTheme.get()).toBe("dark");
  });

  it("tracks system changes only when the preference is system", () => {
    const browser = controlBrowser();
    const theme = createTheme();
    theme.setup();
    browser.dark.change(true);
    expect(theme.colorTheme.get()).toBe("dark");
    expect(document.documentElement.classList.contains("dark")).toBe(true);
    theme.setPreference("light");
    browser.dark.change(false);
    browser.dark.change(true);
    expect(theme.systemTheme.get()).toBe("dark");
    expect(theme.colorTheme.get()).toBe("light");
  });

  it("handles cross-tab storage updates, deletion and clear events", () => {
    controlBrowser({ dark: true });
    const theme = createTheme();
    theme.setup();
    const change = (key: string | null, newValue: string | null) => {
      window.dispatchEvent(
        new StorageEvent("storage", {
          key,
          newValue,
          storageArea: localStorage,
        }),
      );
    };
    change("unrelated", "light");
    expect(theme.colorTheme.get()).toBe("dark");
    window.dispatchEvent(
      new StorageEvent("storage", {
        key: "prefers-color-scheme",
        newValue: "light",
        storageArea: sessionStorage,
      }),
    );
    expect(theme.colorTheme.get()).toBe("dark");
    change("prefers-color-scheme", "light");
    expect(theme.colorTheme.get()).toBe("light");
    change("prefers-color-scheme", null);
    expect(theme.preference.get()).toBe("system");
    expect(theme.colorTheme.get()).toBe("dark");
    change("prefers-color-scheme", "invalid");
    expect(theme.preference.get()).toBe("system");
    change("prefers-color-scheme", "light");
    change(null, null);
    expect(theme.colorTheme.get()).toBe("dark");
  });

  it("uses the actual 767/768 media query for viewport updates", () => {
    const browser = controlBrowser({ mobile: true });
    const theme = createTheme();
    theme.setup();
    expect(theme.isMobile.get()).toBe(true);
    // matchMedia is the source of truth even if an environment has a stale innerWidth.
    browser.mobile.change(false);
    expect(theme.isMobile.get()).toBe(false);
    browser.mobile.change(true);
    expect(theme.isMobile.get()).toBe(true);
    expect(window.matchMedia).toHaveBeenCalledWith("(max-width: 767px)");
  });

  it("falls back when reading storage fails and preserves memory state when writing fails", () => {
    controlBrowser({ dark: true });
    vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => {
      throw new Error("Read denied");
    });
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
      throw new Error("Write denied");
    });
    const theme = createTheme();
    theme.setup();
    expect(theme.preference.get()).toBe("system");
    expect(() => theme.setPreference("light")).not.toThrow();
    expect(theme.colorTheme.get()).toBe("light");
    expect(document.documentElement.classList.contains("light")).toBe(true);
  });

  it("survives a denied localStorage getter", () => {
    controlBrowser({ dark: true });
    vi.spyOn(window, "localStorage", "get").mockImplementation(() => {
      throw new Error("Storage denied");
    });
    const theme = createTheme();
    theme.setup();
    expect(() => theme.setPreference("light")).not.toThrow();
    expect(theme.colorTheme.get()).toBe("light");
  });

  it("has explicit in-memory/system fallbacks without a defaultView", () => {
    const detached = document.implementation.createHTMLDocument("detached");
    expect(detached.defaultView).toBe(null);
    const theme = createTheme(detached);
    expect(() => theme.setup()).not.toThrow();
    expect(theme.colorTheme.get()).toBe("light");
    expect(theme.isMobile.get()).toBe(false);
    theme.setPreference("dark");
    expect(theme.preference.get()).toBe("dark");
    expect(detached.documentElement.classList.contains("dark")).toBe(true);
  });

  it("starts listeners once, removes them on dispose and supports a fresh scope", () => {
    const browser = controlBrowser();
    const darkAdd = vi.spyOn(browser.dark, "addEventListener");
    const darkRemove = vi.spyOn(browser.dark, "removeEventListener");
    const mobileAdd = vi.spyOn(browser.mobile, "addEventListener");
    const mobileRemove = vi.spyOn(browser.mobile, "removeEventListener");
    const windowAdd = vi.spyOn(window, "addEventListener");
    const windowRemove = vi.spyOn(window, "removeEventListener");
    const theme = createTheme();
    theme.setup();
    theme.setup();
    expect(darkAdd).toHaveBeenCalledTimes(1);
    expect(mobileAdd).toHaveBeenCalledTimes(1);
    expect(
      windowAdd.mock.calls.filter(([name]) => name === "storage"),
    ).toHaveLength(1);
    theme.dispose();
    theme.dispose();
    theme.setup();
    expect(darkRemove).toHaveBeenCalledTimes(1);
    expect(mobileRemove).toHaveBeenCalledTimes(1);
    expect(
      windowRemove.mock.calls.filter(([name]) => name === "storage"),
    ).toHaveLength(1);
    browser.dark.change(true);
    browser.mobile.change(true);
    window.dispatchEvent(
      new StorageEvent("storage", {
        key: "prefers-color-scheme",
        newValue: "dark",
      }),
    );
    expect(theme.preference.get()).toBe("system");
    expect(theme.colorTheme.get()).toBe("light");
    expect(theme.isMobile.get()).toBe(false);
    const fresh = createTheme();
    fresh.setup();
    expect(fresh.colorTheme.get()).toBe("dark");
    expect(fresh.isMobile.get()).toBe(true);
    expect(darkAdd).toHaveBeenCalledTimes(2);
    expect(mobileAdd).toHaveBeenCalledTimes(2);
  });
});
