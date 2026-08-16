import {
  createComputed,
  createSignal,
  readonlySignal,
} from "@securitydept/client";
import { inject } from "injection-js";
import { fromEvent, Subscription } from "rxjs";
import { DOCUMENT } from "@/infra/platform/injection";
import { LocalStorageService } from "@/infra/storage/web-storage.service";

export type PreferColorSchemaType = "dark" | "light" | "system";
export type PreferColorSchemaClass = "dark" | "light";

const STORAGE_KEY = "prefers-color-scheme";
const MOBILE_QUERY = "(max-width: 767px)";
const SYSTEM_THEME_QUERY = "(prefers-color-scheme: dark)";

export class ThemeService implements Disposable {
  private readonly document = inject(DOCUMENT);
  private readonly localStorage = inject(LocalStorageService);
  private readonly systemQuery =
    this.document.defaultView?.matchMedia?.(SYSTEM_THEME_QUERY);
  private readonly mobileQuery =
    this.document.defaultView?.matchMedia?.(MOBILE_QUERY);
  private readonly preferenceState = createSignal<PreferColorSchemaType>(
    this.readPreference(),
  );
  private readonly systemState = createSignal<PreferColorSchemaClass>(
    this.systemQuery?.matches ? "dark" : "light",
  );
  private readonly mobileState = createSignal(
    this.mobileQuery?.matches ??
      (this.document.defaultView?.innerWidth ?? 768) < 768,
  );
  private readonly subscriptions = new Subscription();
  private started = false;
  private disposed = false;

  readonly preference = readonlySignal(this.preferenceState);
  readonly systemTheme = readonlySignal(this.systemState);
  readonly isMobile = readonlySignal(this.mobileState);
  readonly colorTheme = createComputed(() => {
    const preference = this.preferenceState.get();
    return preference === "system" ? this.systemState.get() : preference;
  });

  setup(): void {
    if (this.started || this.disposed) return;
    this.started = true;
    this.applyTheme();
    if (this.systemQuery) {
      this.subscriptions.add(
        fromEvent(this.systemQuery, "change").subscribe(() => {
          this.systemState.set(this.systemQuery?.matches ? "dark" : "light");
          this.applyTheme();
        }),
      );
    }
    if (this.mobileQuery) {
      this.subscriptions.add(
        fromEvent(this.mobileQuery, "change").subscribe(() => {
          this.mobileState.set(this.mobileQuery?.matches ?? false);
        }),
      );
    }
    const window = this.document.defaultView;
    if (window) {
      this.subscriptions.add(
        fromEvent<StorageEvent>(window, "storage").subscribe((event) => {
          // A null key represents localStorage.clear(), which resets the preference.
          if (event.key !== STORAGE_KEY && event.key !== null) return;
          try {
            if (event.storageArea && event.storageArea !== window.localStorage)
              return;
          } catch {
            return;
          }
          this.preferenceState.set(this.parsePreference(event.newValue));
          this.applyTheme();
        }),
      );
    }
  }

  setPreference(preference: PreferColorSchemaType): void {
    if (this.disposed) return;
    this.preferenceState.set(preference);
    this.applyTheme();
    try {
      this.localStorage.setItem(STORAGE_KEY, preference);
    } catch {
      // Browser privacy settings can deny persistence; the in-memory preference still works.
    }
  }

  [Symbol.dispose](): void {
    this.dispose();
  }

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    this.subscriptions.unsubscribe();
  }

  private readPreference(): PreferColorSchemaType {
    try {
      return this.parsePreference(this.localStorage.getItem(STORAGE_KEY));
    } catch {
      return "system";
    }
  }

  private parsePreference(value: string | null): PreferColorSchemaType {
    return value === "dark" || value === "light" ? value : "system";
  }

  private applyTheme(): void {
    this.document.documentElement.classList.remove("dark", "light");
    this.document.documentElement.classList.add(this.colorTheme.get());
  }
}
