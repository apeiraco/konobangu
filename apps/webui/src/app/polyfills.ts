export async function ensureTemporal(
  capabilities: { Temporal?: unknown } = globalThis,
  load: () => Promise<unknown> = () => import("temporal-polyfill/global"),
) {
  if (capabilities.Temporal === undefined) await load();
}

export async function ensureResourceManagement(
  capabilities: {
    Symbol: { dispose?: symbol; asyncDispose?: symbol };
    DisposableStack?: unknown;
    AsyncDisposableStack?: unknown;
    SuppressedError?: unknown;
  } = globalThis,
  load: () => Promise<unknown> = () => import("./resource-polyfill"),
) {
  if (
    capabilities.Symbol.dispose === undefined ||
    capabilities.Symbol.asyncDispose === undefined ||
    capabilities.DisposableStack === undefined ||
    capabilities.AsyncDisposableStack === undefined ||
    capabilities.SuppressedError === undefined
  )
    await load();
}

export async function ensurePolyfills(
  loaders: readonly (() => Promise<unknown>)[] = [
    ensureTemporal,
    ensureResourceManagement,
  ],
) {
  await Promise.all(loaders.map((load) => load()));
}
