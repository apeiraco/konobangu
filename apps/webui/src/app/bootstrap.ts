import { ensurePolyfills } from "./polyfills";

export function createBootstrap(
  loadApp: () => Promise<{ startApp: () => Disposable }>,
  ensure: () => Promise<void> = ensurePolyfills,
) {
  let pending: Promise<void> | undefined;
  let lifetime: DisposableStack | undefined;
  let disposed = false;
  const dispose = () => {
    if (disposed) return;
    disposed = true;
    const owner = lifetime;
    lifetime = undefined;
    owner?.[Symbol.dispose]();
  };
  return {
    // This controller exists before polyfills; only post-barrier owners are Disposable.
    start() {
      if (disposed || lifetime) return Promise.resolve();
      if (pending) return pending;
      pending = (async () => {
        await ensure();
        if (disposed) return;
        const entry = await loadApp();
        if (disposed) return;
        using ownership = new DisposableStack();
        ownership.use(entry.startApp());
        // A startup callback can synchronously invalidate this bootstrap.
        if (!disposed) lifetime = ownership.move();
      })().finally(() => {
        // Settle after assignment, even when a capability loader throws synchronously.
        pending = undefined;
      });
      return pending;
    },
    dispose,
  };
}
