// @vitest-environment jsdom
import { ApolloProvider } from "@apollo/client/react";
import { createCancellationTokenSource } from "@securitydept/client";
import { SecuritydeptProvider } from "@securitydept/client-react";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { toast } from "sonner";
import { afterEach, describe, expect, it, vi } from "vitest";
import {
  createSessionScope,
  response,
  sessionPayload,
} from "@/__test__/support/auth";
import {
  GET_CREDENTIAL_3RD,
  GET_CREDENTIAL_3RD_DETAIL,
} from "@/domains/recorder/schema/credential3rd";
import { GraphQLService } from "@/infra/graphql/graphql.service";
import { Credential3rdCheckAvailableView } from "@/presentation/routes/_app/credential3rd/-check-available";
import { Credential3rdService } from "../credential3rd.service";

const scopes: ReturnType<typeof createSessionScope>[] = [];
afterEach(() => {
  cleanup();
  for (const scope of scopes.splice(0)) scope.runtime.dispose();
  vi.restoreAllMocks();
  vi.unstubAllEnvs();
});
async function fixture(command: Parameters<typeof createSessionScope>[0]) {
  vi.stubEnv("AUTH__PROVIDER__TYPE", "oidc");
  let subject: string | null = "A";
  const scope = createSessionScope((request) =>
    request.method === "POST"
      ? command(request)
      : response(subject ? 200 : 401, subject ? sessionPayload(subject) : null),
  );
  scopes.push(scope);
  await scope.auth.session.start();
  return {
    ...scope,
    service: scope.runtime.injector.get(Credential3rdService),
    identity: async (next: string | null) => {
      subject = next;
      await scope.auth.session.refresh();
    },
  };
}
describe("Independent credential command", () => {
  it("uses the shared SDK transport with a JSON body and CSRF header", async () => {
    const scope = await fixture(() => response(200, { available: true }));
    expect(await scope.service.checkAvailable(42)).toEqual({ available: true });
    expect(scope.execute).toHaveBeenLastCalledWith(
      expect.objectContaining({
        url: "http://localhost:3000/api/credential3rd/42/check-available",
        method: "POST",
        body: {},
        headers: {
          "Content-Type": "application/json",
          "X-Konobangu-CSRF": "1",
        },
      }),
    );
  });
  it.each([403, 409, 502])(
    "preserves HTTP %i as a failure without changing the identity",
    async (status) => {
      const scope = await fixture(() =>
        response(status, { message: "command failed" }),
      );
      await expect(scope.service.checkAvailable(42)).rejects.toThrow(
        "command failed",
      );
      expect(scope.auth.isAuthenticated.get()).toBe(true);
      expect(scope.execute).toHaveBeenCalledTimes(2);
    },
  );
  it.each([null, { available: "true" }])(
    "rejects malformed command success %j",
    async (body) => {
      const scope = await fixture(() => response(200, body));
      await expect(scope.service.checkAvailable(42)).rejects.toThrow(
        "Invalid credential check response",
      );
    },
  );
  it.each(["caller", "identity", "root"])(
    "rejects late replies after %s cancellation",
    async (mode) => {
      let complete: ((result: ReturnType<typeof response>) => void) | undefined;
      const scope = await fixture(
        () =>
          new Promise((resolve) => {
            complete = resolve;
          }),
      );
      const source = createCancellationTokenSource();
      const pending = scope.service
        .checkAvailable(42, source.token)
        .catch((error: unknown) => error);
      await vi.waitFor(() => expect(complete).toBeDefined());
      if (mode === "caller") source.cancel();
      if (mode === "identity") await scope.identity("B");
      if (mode === "root") scope.runtime.dispose();
      const request = scope.execute.mock.calls.find(
        ([request]) => request.method === "POST",
      )?.[0];
      await vi.waitFor(() =>
        expect(request?.cancellationToken?.isCancellationRequested).toBe(true),
      );
      complete?.(response(200, { available: true }));
      expect(await pending).toMatchObject({ name: "AbortError" });
    },
  );
  it("refreshes the SDK session after 401", async () => {
    const scope = await fixture(() => response(401));
    await scope.identity(null);
    const before = scope.execute.mock.calls.length;
    await expect(scope.service.checkAvailable(42)).rejects.toThrow();
    await vi.waitFor(() =>
      expect(scope.execute.mock.calls.length).toBe(before + 2),
    );
    expect(scope.auth.isAuthenticated.get()).toBe(false);
  });
});
describe("Credential check button", () => {
  async function mount(command: Parameters<typeof createSessionScope>[0]) {
    const scope = await fixture(command);
    const graphql = scope.runtime.injector.get(GraphQLService);
    scope.runtime.resources.use(graphql);
    const refetch = vi
      .spyOn(graphql._apollo, "refetchQueries")
      .mockResolvedValue([]);
    const view = render(
      <SecuritydeptProvider injector={scope.runtime.facade}>
        <ApolloProvider client={graphql._apollo}>
          <Credential3rdCheckAvailableView id={42} />
        </ApolloProvider>
      </SecuritydeptProvider>,
    );
    return { ...scope, ...view, refetch };
  }
  it("disables during the command and refreshes the active Apollo credential queries after success", async () => {
    let complete: ((result: ReturnType<typeof response>) => void) | undefined;
    const success = vi.spyOn(toast, "success");
    const scope = await mount(
      () =>
        new Promise((resolve) => {
          complete = resolve;
        }),
    );
    const button = screen.getByRole("button", { name: "Check Available" });
    fireEvent.click(button);
    expect(button.hasAttribute("disabled")).toBe(true);
    await vi.waitFor(() => expect(complete).toBeDefined());
    complete?.(response(200, { available: true }));
    await vi.waitFor(() => expect(button.hasAttribute("disabled")).toBe(false));
    expect(scope.refetch).toHaveBeenCalledWith({
      include: [GET_CREDENTIAL_3RD, GET_CREDENTIAL_3RD_DETAIL],
    });
    expect(success).toHaveBeenCalledWith("Credential is available");
    expect(
      scope.execute.mock.calls.filter(([request]) => request.method === "POST"),
    ).toHaveLength(1);
  });
  it("shows the command error and does not refetch after a conflict", async () => {
    const failure = vi.spyOn(toast, "error");
    const scope = await mount(() =>
      response(409, { message: "Credential changed" }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Check Available" }));
    await vi.waitFor(() =>
      expect(failure).toHaveBeenCalledWith("Failed to check available", {
        description: "Credential changed",
      }),
    );
    expect(scope.refetch).not.toHaveBeenCalled();
  });
  it("cancels on unmount and prevents late notification and refetch", async () => {
    let complete: ((result: ReturnType<typeof response>) => void) | undefined;
    const success = vi.spyOn(toast, "success");
    const scope = await mount(
      () =>
        new Promise((resolve) => {
          complete = resolve;
        }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Check Available" }));
    await vi.waitFor(() => expect(complete).toBeDefined());
    scope.unmount();
    complete?.(response(200, { available: true }));
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(scope.refetch).not.toHaveBeenCalled();
    expect(success).not.toHaveBeenCalled();
  });
});
