// @vitest-environment jsdom
import {
  createCancellationTokenSource,
  ENVIRONMENT_TOKEN,
  INJECTOR_TOKEN,
} from "@securitydept/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import {
  createSessionScope,
  response,
  sessionPayload,
} from "@/__test__/support/auth";

const scopes: ReturnType<typeof createSessionScope>[] = [];
afterEach(() => {
  for (const scope of scopes.splice(0)) scope.runtime.dispose();
  vi.unstubAllEnvs();
});
function scope(handler: Parameters<typeof createSessionScope>[0]) {
  vi.stubEnv("AUTH__PROVIDER__TYPE", "oidc");
  const scope = createSessionScope(handler);
  scopes.push(scope);
  return scope;
}

describe("SDK-owned application session", () => {
  it("disposes SDK work with the root and rejects a late response", async () => {
    let finish: ((result: ReturnType<typeof response>) => void) | undefined;
    const { runtime, auth } = scope(
      () =>
        new Promise((resolve) => {
          finish = resolve;
        }),
    );
    const started = auth.session.start().catch((error: unknown) => error);
    await vi.waitFor(() => expect(finish).toBeDefined());
    runtime.dispose();
    expect(runtime.destroyRef.destroyed).toBe(true);
    finish?.(response(200, sessionPayload("late")));
    await started;
    expect(auth.isAuthenticated.get()).toBe(false);
  });
  it("uses one foundation root and probes the server only once when started repeatedly", async () => {
    const { runtime, auth, execute } = scope(() =>
      response(200, sessionPayload()),
    );
    expect(runtime.injector.get(ENVIRONMENT_TOKEN)).toBe(runtime.environment);
    expect(runtime.injector.get(INJECTOR_TOKEN)).toBe(runtime.injector);
    await Promise.all([auth.session.start(), auth.session.start()]);
    expect(execute).toHaveBeenCalledTimes(1);
    expect(auth.isAuthenticated.get()).toBe(true);
    expect(auth.userData.get()?.principal.subject).toBe("A");
  });
  it("uses the server result in Basic mode and explains the credential-cache logout limit", async () => {
    const state = scope(() => response(401));
    vi.stubEnv("AUTH__PROVIDER__TYPE", "basic");
    const basic = createSessionScope(() => response(401));
    scopes.push(basic);
    await basic.auth.session.start();
    expect(basic.auth.isAuthenticated.get()).toBe(false);
    await expect(basic.auth.logout()).rejects.toThrow(
      "browser caches Basic credentials",
    );
    expect(state.auth.isAuthenticated.get()).toBe(false);
  });
  it.each([403, 500, 503])(
    "preserves HTTP %i as a failure, avoiding login loops",
    async (status) => {
      const { auth } = scope(() => response(status, { message: "denied" }));
      await expect(auth.session.start()).rejects.toThrow();
      expect(auth.check.get().status).toBe("error");
      expect(auth.isAuthenticated.get()).toBe(false);
    },
  );
  it("distinguishes an unauthenticated response from malformed success", async () => {
    const anonymous = scope(() => response(401));
    expect(await anonymous.auth.session.start()).toBeNull();
    expect(anonymous.auth.check.get()).toEqual({
      status: "success",
      isAuthenticated: false,
    });
    const malformed = scope(() =>
      response(200, { principal: { subject: 42 } }),
    );
    await expect(malformed.auth.session.start()).rejects.toThrow();
    expect(malformed.auth.check.get().status).toBe("error");
  });
  it("refreshes the real SDK resource and clears it after successful logout", async () => {
    let signedIn = true;
    const { auth, execute } = scope((request) => {
      if (request.method === "POST") {
        signedIn = false;
        return response(200, { success: true });
      }
      return signedIn ? response(200, sessionPayload()) : response(401);
    });
    await auth.session.start();
    await auth.session.refresh();
    expect(auth.isAuthenticated.get()).toBe(true);
    await auth.logout();
    expect(auth.isAuthenticated.get()).toBe(false);
    expect(auth.userData.get()).toBeNull();
    expect(execute).toHaveBeenCalledTimes(3);
  });
  it("cancels an operation through the public cancellation contract", async () => {
    const { auth } = scope(
      (request) =>
        new Promise((resolve) => {
          request.cancellationToken?.onCancellationRequested(() =>
            resolve(response(200, sessionPayload("late"))),
          );
        }),
    );
    const cancellation = createCancellationTokenSource();
    const refresh = auth.session.refresh({
      cancellationToken: cancellation.token,
    });
    cancellation.cancel();
    await expect(refresh).rejects.toThrow();
    expect(auth.userData.get()).toBeNull();
  });
});
