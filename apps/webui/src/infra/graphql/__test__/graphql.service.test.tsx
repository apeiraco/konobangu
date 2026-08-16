// @vitest-environment jsdom
import { gql } from "@apollo/client";
import { ApolloProvider, useQuery } from "@apollo/client/react";
import type { TypedDocumentNode } from "@graphql-typed-document-node/core";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import {
  createSessionScope,
  response,
  sessionPayload,
} from "@/__test__/support/auth";
import { QueryPartialError } from "@/components/ui/query-error-view";
import { GraphQLService } from "../graphql.service";

const query: TypedDocumentNode<
  { privateName: string },
  Record<string, never>
> = gql`query PrivateName { privateName }`;
const mutation: TypedDocumentNode<
  { updatePrivateName: string },
  Record<string, never>
> = gql`mutation UpdatePrivateName { updatePrivateName }`;
const scopes: ReturnType<typeof createSessionScope>[] = [];
afterEach(() => {
  cleanup();
  for (const scope of scopes.splice(0)) scope.runtime.dispose();
  vi.unstubAllGlobals();
  vi.unstubAllEnvs();
});
async function fixture() {
  vi.stubEnv("AUTH__PROVIDER__TYPE", "oidc");
  let subject: string | null = "A";
  const scope = createSessionScope(() =>
    subject ? response(200, sessionPayload(subject)) : response(401),
  );
  scopes.push(scope);
  await scope.auth.session.start();
  const graphql = scope.runtime.injector.get(GraphQLService);
  scope.runtime.resources.use(graphql);
  return {
    ...scope,
    graphql,
    identity: async (value: string | null) => {
      subject = value;
      await scope.auth.session.refresh();
    },
  };
}
describe("Apollo identity scopes and typed errors", () => {
  it("keeps the cache for a same-identity refresh and replaces it on logout or issuer/subject change", async () => {
    const scope = await fixture();
    const first = scope.graphql._apollo;
    first.writeQuery({ query, data: { privateName: "A private" } });
    let complete: ((result: ReturnType<typeof response>) => void) | undefined;
    scope.execute.mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          complete = resolve;
        }),
    );
    const refreshing = scope.auth.session.refresh();
    await waitFor(() => expect(complete).toBeDefined());
    expect(scope.auth.isAuthenticated.get()).toBe(true);
    expect(scope.graphql._apollo).toBe(first);
    expect(first.readQuery({ query })).toEqual({ privateName: "A private" });
    complete?.(response(200, sessionPayload("A")));
    await refreshing;
    expect(scope.graphql._apollo).toBe(first);
    await scope.identity("B");
    const second = scope.graphql._apollo;
    expect(second).not.toBe(first);
    expect(second.readQuery({ query })).toBeNull();
    second.writeQuery({ query, data: { privateName: "B private" } });
    await scope.identity(null);
    expect(scope.graphql._apollo.readQuery({ query })).toBeNull();
  });
  it("aborts old requests and prevents a late response from filling the next identity cache", async () => {
    const scope = await fixture();
    let complete: ((response: Response) => void) | undefined;
    let signal: AbortSignal | null | undefined;
    vi.stubGlobal(
      "fetch",
      vi.fn((_input: RequestInfo | URL, options?: RequestInit) => {
        signal = options?.signal;
        return new Promise<Response>((resolve) => {
          complete = resolve;
        });
      }),
    );
    const old = scope.graphql.query({ query }).catch((error: unknown) => error);
    await waitFor(() => expect(complete).toBeDefined());
    await scope.identity("B");
    expect(signal?.aborted).toBe(true);
    complete?.(
      new Response(JSON.stringify({ data: { privateName: "late A" } }), {
        headers: { "content-type": "application/json" },
      }),
    );
    await old;
    expect(scope.graphql._apollo.readQuery({ query })).toBeNull();
  });
  it("displays partial query data beside errors, while mutation errors reject", async () => {
    const scope = await fixture();
    vi.stubGlobal(
      "fetch",
      vi.fn(
        async () =>
          new Response(
            JSON.stringify({
              data: { privateName: "visible partial data" },
              errors: [{ message: "another field failed" }],
            }),
            { headers: { "content-type": "application/json" } },
          ),
      ),
    );
    function Consumer() {
      const { data, error } = useQuery(query);
      return (
        <>
          <span>{data?.privateName}</span>
          <QueryPartialError error={error} />
        </>
      );
    }
    render(
      <ApolloProvider client={scope.graphql._apollo}>
        <Consumer />
      </ApolloProvider>,
    );
    expect(await screen.findByText("visible partial data")).toBeTruthy();
    expect((await screen.findByRole("alert")).textContent).toContain(
      "another field failed",
    );
    await expect(scope.graphql.mutate({ mutation })).rejects.toThrow(
      "another field failed",
    );
  });
});
