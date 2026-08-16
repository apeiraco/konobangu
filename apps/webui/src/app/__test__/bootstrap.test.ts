import { afterEach, describe, expect, it, vi } from "vitest";
import { createBootstrap } from "../bootstrap";
import {
  ensurePolyfills,
  ensureResourceManagement,
  ensureTemporal,
} from "../polyfills";

function deferred() {
  let resolve!: () => void;
  const promise = new Promise<void>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}
const controllers = new Set<ReturnType<typeof createBootstrap>>();
afterEach(() => {
  for (const controller of controllers) controller.dispose();
  controllers.clear();
});
function fixture(...args: Parameters<typeof createBootstrap>) {
  const controller = createBootstrap(...args);
  controllers.add(controller);
  return controller;
}
function application(dispose = vi.fn()) {
  return { startApp: vi.fn(() => ({ [Symbol.dispose]: dispose })) };
}
describe("application startup barrier", () => {
  it("creates and cancels the controller without any resource primitives", async () => {
    const pending = deferred();
    const entry = vi.fn(async () => application());
    let done: Promise<void>;
    let keys: symbol[];
    try {
      // Model the pre-polyfill phase, including an unavailable protocol symbol.
      vi.stubGlobal("Symbol", undefined);
      vi.stubGlobal("DisposableStack", undefined);
      const boot = fixture(entry, () => pending.promise);
      keys = Reflect.ownKeys(boot).filter(
        (key): key is symbol => typeof key === "symbol",
      );
      done = boot.start();
      boot.dispose();
      boot.dispose();
    } finally {
      vi.unstubAllGlobals();
    }
    pending.resolve();
    await done;
    expect(keys).toEqual([]);
    expect(entry).not.toHaveBeenCalled();
  });
  it("preserves native resource management without loading its polyfill", async () => {
    const capabilities = {
      Symbol: { dispose: Symbol.dispose, asyncDispose: Symbol.asyncDispose },
      DisposableStack,
      AsyncDisposableStack,
      SuppressedError,
    };
    const load = vi.fn();
    await ensureResourceManagement(capabilities, load);
    expect(load).not.toHaveBeenCalled();
    expect(capabilities.Symbol.dispose).toBe(Symbol.dispose);
  });
  it("waits for missing resource primitives before evaluating SDK owners", async () => {
    const capabilities: Parameters<typeof ensureResourceManagement>[0] = {
      Symbol: {},
    };
    const pending = deferred();
    const entry = vi.fn(async () => application());
    const boot = fixture(entry, () =>
      ensureResourceManagement(capabilities, () => pending.promise),
    );
    const done = boot.start();
    expect(entry).not.toHaveBeenCalled();
    pending.resolve();
    await done;
    expect(entry).toHaveBeenCalledTimes(1);
  });
  it("skips importing and retains the native object", async () => {
    const native = {};
    const capabilities = { Temporal: native };
    const load = vi.fn();
    await ensureTemporal(capabilities, load);
    expect(load).not.toHaveBeenCalled();
    expect(capabilities.Temporal).toBe(native);
  });
  it("loads missing Temporal before evaluating the app", async () => {
    const capabilities: { Temporal?: unknown } = {};
    const load = vi.fn(async () => {
      capabilities.Temporal = {};
    });
    const entry = vi.fn(async () => {
      expect(capabilities.Temporal).toBeDefined();
      return application();
    });
    const boot = fixture(entry, () => ensureTemporal(capabilities, load));
    await boot.start();
    expect(load).toHaveBeenCalledTimes(1);
    expect(entry).toHaveBeenCalledTimes(1);
  });
  it("starts independent loaders concurrently and waits for every one", async () => {
    const a = deferred();
    const b = deferred();
    const first = vi.fn(() => a.promise);
    const second = vi.fn(() => b.promise);
    const entry = vi.fn(async () => application());
    const boot = fixture(entry, () => ensurePolyfills([first, second]));
    const done = boot.start();
    expect(first).toHaveBeenCalledTimes(1);
    expect(second).toHaveBeenCalledTimes(1);
    a.resolve();
    await a.promise;
    expect(entry).not.toHaveBeenCalled();
    b.resolve();
    await done;
    expect(entry).toHaveBeenCalledTimes(1);
  });
  it("does not start on failure and retries without duplicate roots or subscriptions", async () => {
    const ensure = vi
      .fn()
      .mockRejectedValueOnce(new Error("offline"))
      .mockResolvedValue(undefined);
    const stop = vi.fn();
    const { startApp } = application(stop);
    const entry = vi.fn(async () => ({ startApp }));
    const boot = fixture(entry, ensure);
    await expect(boot.start()).rejects.toThrow("offline");
    expect(entry).not.toHaveBeenCalled();
    await Promise.all([boot.start(), boot.start()]);
    await boot.start();
    expect(startApp).toHaveBeenCalledTimes(1);
    boot.dispose();
    boot.dispose();
    expect(stop).toHaveBeenCalledTimes(1);
  });
  it("HMR disposal prevents pending initialization and a replacement owns one lifecycle", async () => {
    const pending = deferred();
    const entry = vi.fn(async () => application());
    const boot = fixture(entry, () => pending.promise);
    const done = boot.start();
    boot.dispose();
    pending.resolve();
    await done;
    expect(entry).not.toHaveBeenCalled();
    const replacement = fixture(entry);
    await replacement.start();
    expect(entry).toHaveBeenCalledTimes(1);
    replacement.dispose();
  });
  it("retains post-barrier application ownership until controller disposal", async () => {
    const dispose = vi.fn();
    const boot = fixture(async () => application(dispose));
    await boot.start();
    expect(dispose).not.toHaveBeenCalled();
    boot.dispose();
    expect(dispose).toHaveBeenCalledTimes(1);
  });
  it("does not retry cleanup or startup after application disposal throws", async () => {
    const error = new Error("unmount failed");
    const dispose = vi.fn(() => {
      throw error;
    });
    const entry = vi.fn(async () => application(dispose));
    const boot = fixture(entry);
    await boot.start();
    expect(() => boot.dispose()).toThrow(error);
    boot.dispose();
    await boot.start();
    expect(dispose).toHaveBeenCalledTimes(1);
    expect(entry).toHaveBeenCalledTimes(1);
  });
  it("retries a capability loader that throws before returning a promise", async () => {
    const ensure = vi
      .fn()
      .mockImplementationOnce(() => {
        throw new Error("capability check failed");
      })
      .mockResolvedValue(undefined);
    const entry = vi.fn(async () => application());
    const boot = fixture(entry, ensure);
    await expect(boot.start()).rejects.toThrow("capability check failed");
    await boot.start();
    expect(ensure).toHaveBeenCalledTimes(2);
    expect(entry).toHaveBeenCalledTimes(1);
  });
  it("does not mount an entry that resolves after disposal", async () => {
    const pending = deferred();
    const app = application();
    const entry = vi.fn(async () => {
      await pending.promise;
      return app;
    });
    const boot = fixture(entry);
    const done = boot.start();
    await vi.waitFor(() => expect(entry).toHaveBeenCalledTimes(1));
    boot.dispose();
    pending.resolve();
    await done;
    expect(app.startApp).not.toHaveBeenCalled();
  });
  it("releases a root when startup synchronously disposes the bootstrap", async () => {
    const dispose = vi.fn();
    const boot = fixture(async () => ({
      startApp() {
        boot.dispose();
        return { [Symbol.dispose]: dispose };
      },
    }));
    await boot.start();
    expect(dispose).toHaveBeenCalledTimes(1);
  });
});
