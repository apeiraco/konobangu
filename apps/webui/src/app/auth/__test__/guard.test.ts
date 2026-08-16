// @vitest-environment jsdom
import type { ParsedLocation } from "@tanstack/react-router";
import { afterEach, describe, expect, it, vi } from "vitest";
import {
  createSessionScope,
  response,
  sessionPayload,
} from "@/__test__/support/auth";
import { beforeLoadGuard } from "../guard";

const scopes: ReturnType<typeof createSessionScope>[] = [];
afterEach(() => {
  for (const scope of scopes.splice(0)) scope.runtime.dispose();
  vi.unstubAllEnvs();
});
function fixture(status: number) {
  vi.stubEnv("AUTH__PROVIDER__TYPE", "oidc");
  const scope = createSessionScope(() =>
    response(status, status === 200 ? sessionPayload() : null),
  );
  scopes.push(scope);
  const location: ParsedLocation = {
    href: "/subscriptions",
    publicHref: "/subscriptions",
    pathname: "/subscriptions",
    search: {},
    searchStr: "",
    state: { __TSR_index: 0 },
    hash: "",
    external: false,
  };
  return {
    ...scope,
    args: { context: { injector: scope.runtime.injector }, location },
  };
}
describe("session route guard", () => {
  it("waits for server authentication before allowing private routes", async () => {
    const { args } = fixture(200);
    await expect(beforeLoadGuard(args)).resolves.toBeUndefined();
  });
  it("redirects only a confirmed 401 and preserves the local return path", async () => {
    const { args, auth } = fixture(401);
    const login = vi.spyOn(auth, "login").mockResolvedValue();
    await expect(beforeLoadGuard(args)).rejects.toThrow("redirect started");
    expect(login).toHaveBeenCalledWith("/subscriptions");
  });
  it.each([403, 500])(
    "does not convert HTTP %i into automatic login",
    async (status) => {
      const { args, auth } = fixture(status);
      const login = vi.spyOn(auth, "login");
      await expect(beforeLoadGuard(args)).rejects.toThrow();
      expect(login).not.toHaveBeenCalled();
    },
  );
});
