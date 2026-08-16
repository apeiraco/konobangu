import { vi } from "vitest";

export class ControlledMediaQueryList
  extends EventTarget
  implements MediaQueryList
{
  onchange: MediaQueryList["onchange"] = null;
  private readonly legacyListeners = new Map<
    NonNullable<MediaQueryList["onchange"]>,
    EventListener
  >();

  constructor(
    readonly media: string,
    public matches: boolean,
  ) {
    super();
  }

  change(matches: boolean): void {
    this.matches = matches;
    const event = Object.assign(new Event("change"), {
      media: this.media,
      matches,
    });
    this.onchange?.call(this, event);
    this.dispatchEvent(event);
  }

  addListener(listener: MediaQueryList["onchange"]): void {
    if (!listener || this.legacyListeners.has(listener)) return;
    const handler = () =>
      listener.call(
        this,
        Object.assign(new Event("change"), {
          media: this.media,
          matches: this.matches,
        }),
      );
    this.legacyListeners.set(listener, handler);
    this.addEventListener("change", handler);
  }

  removeListener(listener: MediaQueryList["onchange"]): void {
    if (!listener) return;
    const handler = this.legacyListeners.get(listener);
    if (handler) this.removeEventListener("change", handler);
    this.legacyListeners.delete(listener);
  }
}

export function controlBrowser(
  options: { dark?: boolean; mobile?: boolean } = {},
) {
  const dark = new ControlledMediaQueryList(
    "(prefers-color-scheme: dark)",
    options.dark ?? false,
  );
  const mobile = new ControlledMediaQueryList(
    "(max-width: 767px)",
    options.mobile ?? false,
  );
  vi.stubGlobal(
    "matchMedia",
    vi.fn((query: string): MediaQueryList => {
      if (query === dark.media) return dark;
      if (query === mobile.media) return mobile;
      return new ControlledMediaQueryList(query, false);
    }),
  );
  return { dark, mobile };
}
