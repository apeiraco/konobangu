// @vitest-environment jsdom
import { ApolloProvider } from "@apollo/client/react";
import { SecuritydeptProvider } from "@securitydept/client-react";
import {
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
  RouterProvider,
} from "@tanstack/react-router";
import { useTable } from "@tanstack/react-table";
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import type { ComponentType } from "react";
import { toast } from "sonner";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  createSessionScope,
  response,
  sessionPayload,
} from "@/__test__/support/auth";
import { GraphQLService } from "@/infra/graphql/graphql.service";
import { CredentialManageRouteComponent } from "@/presentation/routes/_app/credential3rd/manage";
import { SubscriptionManageRouteComponent } from "@/presentation/routes/_app/subscriptions/manage";
import { TaskCronManageRouteComponent } from "@/presentation/routes/_app/tasks/cron/manage";
import { TaskManageRouteComponent } from "@/presentation/routes/_app/tasks/manage";
import { DataTableColumnHeader } from "../data-table-column-header";
import { DataTablePagination } from "../data-table-pagination";
import { useClampServerPage, useServerTableState } from "../data-table-state";

beforeEach(() => {
  vi.spyOn(window, "scrollTo").mockImplementation(() => {});
  HTMLElement.prototype.scrollIntoView = vi.fn();
});
const scopes: ReturnType<typeof createSessionScope>[] = [];
afterEach(() => {
  cleanup();
  for (const scope of scopes.splice(0)) scope.runtime.dispose();
  vi.unstubAllGlobals();
  vi.unstubAllEnvs();
  vi.restoreAllMocks();
});
const cases = [
  ["subscriptions", SubscriptionManageRouteComponent, "displayName"],
  ["credential3rd", CredentialManageRouteComponent, "userAgent"],
  ["subscriberTasks", TaskManageRouteComponent, "id"],
  ["cron", TaskCronManageRouteComponent, "cronExpr"],
] as const;
interface Variables {
  pagination: { page: { page: number; limit: number } };
  orderBy: Record<string, string>;
  filter: Record<string, unknown>;
}
const record = {
  id: "server-row",
  displayName: "server row",
  username: "fixture-user",
  password: "",
  cookies: "",
  credentialType: "MIKAN",
  userAgent: "fixture-agent",
  category: "MIKAN_SUBSCRIBER",
  sourceUrl: "https://fixture.test/source",
  enabled: true,
  createdAt: "2030-01-01T00:00:00Z",
  updatedAt: "2030-01-01T00:00:00Z",
  job: { task_type: "sync_subscription_sources" },
  taskType: "SYNC_SUBSCRIPTION_SOURCES",
  status: "FAILED",
  attempts: 1,
  maxAttempts: 3,
  generation: 1,
  runAt: "2030-01-01T00:00:00Z",
  doneAt: null,
  lastError: "fixture error",
  cancelRequestedAt: null,
  subscription: null,
  cron: null,
  cronExpr: "0 * * * * *",
  cronTimezone: "UTC",
  nextRun: "2026-10-04 19:38:05.559928 UTC",
  lastRun: "2026-10-04 12:38:05.559928 -07:00",
  lockedAt: "invalid",
  lockedBy: null,
  timeoutMs: null,
  priority: 0,
  subscriberTaskCron: null,
  systemTaskCron: null,
  subscriberId: 1,
};
async function mount(Page: ComponentType, field: string, partial = false) {
  vi.stubEnv("AUTH__PROVIDER__TYPE", "oidc");
  const scope = createSessionScope(() => response(200, sessionPayload()));
  scopes.push(scope);
  await scope.auth.session.start();
  const graphql = scope.runtime.injector.get(GraphQLService);
  const requests: Variables[] = [];
  let pages = 3;
  let mutationError: string | null = null;
  let mutationCount = 1;
  vi.stubGlobal(
    "fetch",
    vi.fn(async (_input: RequestInfo | URL, init?: RequestInit) => {
      const body = JSON.parse(String(init?.body)) as {
        query: string;
        variables: Variables;
      };
      if (body.query.includes("mutation")) {
        return new Response(
          JSON.stringify(
            mutationError
              ? { data: null, errors: [{ message: mutationError }] }
              : { data: { subscriberTasksDelete: mutationCount } },
          ),
          { headers: { "content-type": "application/json" } },
        );
      }
      requests.push(body.variables);
      const nodes =
        body.variables.pagination.page.page >= pages
          ? []
          : [
              {
                ...record,
                id: field === "subscriberTasks" ? "server-row" : 17,
              },
            ];
      return new Response(
        JSON.stringify({
          data: { [field]: { nodes, paginationInfo: { total: 21, pages } } },
          ...(partial ? { errors: [{ message: "another field failed" }] } : {}),
        }),
        { headers: { "content-type": "application/json" } },
      );
    }),
  );
  const root = createRootRoute();
  const route = createRoute({
    getParentRoute: () => root,
    path: "/",
    component: () => <Page />,
  });
  const router = createRouter({
    routeTree: root.addChildren([route]),
    history: createMemoryHistory({ initialEntries: ["/"] }),
  });
  render(
    <SecuritydeptProvider injector={scope.runtime.injector}>
      <ApolloProvider client={graphql._apollo}>
        <RouterProvider router={router} />
      </ApolloProvider>
    </SecuritydeptProvider>,
  );
  await screen.findByLabelText("Filter records");
  await waitFor(() => expect(requests.length).toBeGreaterThan(0));
  return {
    requests,
    empty: () => {
      pages = 1;
    },
    mutationFailure: (error: string) => {
      mutationError = error;
    },
    mutationZero: () => {
      mutationCount = 0;
    },
    client: graphql._apollo,
  };
}
function openMenu(button: HTMLElement) {
  fireEvent.pointerDown(button, {
    button: 0,
    ctrlKey: false,
    pointerType: "mouse",
  });
  fireEvent.keyDown(button, { key: "ArrowDown" });
}
describe("Native Table V9 actual management pages", () => {
  it.each(cases)(
    "%s sends server pagination, sorting and filters without paging the returned page again",
    async (field, Page, filterField) => {
      const fixture = await mount(Page, field);
      fireEvent.click(screen.getByRole("button", { name: "Go to next page" }));
      await waitFor(() =>
        expect(fixture.requests.at(-1)?.pagination.page.page).toBe(1),
      );
      await waitFor(() =>
        expect(screen.getAllByRole("button", { name: "Detail" }).length).toBe(
          1,
        ),
      );
      fireEvent.change(screen.getByLabelText("Filter records"), {
        target: { value: "search" },
      });
      await waitFor(() =>
        expect(fixture.requests.at(-1)?.filter).toEqual({
          [filterField]: { contains: "search" },
        }),
      );
      expect(fixture.requests.at(-1)?.pagination.page.page).toBe(0);
      fireEvent.click(screen.getByRole("button", { name: "Go to next page" }));
      await waitFor(() =>
        expect(fixture.requests.at(-1)?.pagination.page.page).toBe(1),
      );
      openMenu(
        screen.getByRole("button", {
          name:
            field === "subscriberTasks" || field === "cron"
              ? "Sort by ID"
              : field === "subscriptions"
                ? "Name"
                : "ID",
        }),
      );
      fireEvent.click(await screen.findByRole("menuitem", { name: "Asc" }));
      await waitFor(() =>
        expect(fixture.requests.at(-1)?.orderBy).toEqual(
          field === "subscriptions" ? { displayName: "ASC" } : { id: "ASC" },
        ),
      );
      expect(fixture.requests.at(-1)?.pagination.page.page).toBe(0);
      fireEvent.click(screen.getByRole("button", { name: "Go to last page" }));
      await waitFor(() =>
        expect(fixture.requests.at(-1)?.pagination.page.page).toBe(2),
      );
      fixture.empty();
      await fixture.client.refetchQueries({ include: "active" });
      await waitFor(() =>
        expect(fixture.requests.at(-1)?.pagination.page.page).toBe(0),
      );
    },
  );
  it("keeps partial data visible beside GraphQL errors", async () => {
    await mount(TaskManageRouteComponent, "subscriberTasks", true);
    expect(await screen.findByText(/# server-row/)).toBeTruthy();
    expect((await screen.findByRole("alert")).textContent).toContain(
      "another field failed",
    );
  });
  it.each(["conflict", "zero"])(
    "task %s rejection never shows successful cancellation",
    async (mode) => {
      const fixture = await mount(TaskManageRouteComponent, "subscriberTasks");
      const success = vi.spyOn(toast, "success");
      const failure = vi.spyOn(toast, "error");
      if (mode === "zero") fixture.mutationZero();
      else fixture.mutationFailure("Task conflict");
      openMenu(await screen.findByRole("button", { name: "Open menu" }));
      fireEvent.click(await screen.findByRole("menuitem", { name: /Delete/ }));
      await waitFor(() => expect(failure).toHaveBeenCalled());
      expect(success).not.toHaveBeenCalled();
    },
  );
});
function SharedTable() {
  const state = useServerTableState();
  const table = useTable({
    ...state.tableOptions,
    data: [{ id: "stable-id", name: "visible" }],
    columns: [{ accessorKey: "name" }],
    pageCount: 3,
    rowCount: 21,
  });
  useClampServerPage(state, 3, false);
  return (
    <>
      <DataTableColumnHeader column={table.getColumn("name")!} title="Name" />
      <button
        type="button"
        onClick={() => table.getRow("stable-id").toggleSelected()}
      >
        Select row
      </button>
      <output>{JSON.stringify(table.state.rowSelection)}</output>
      <output>{String(table.getColumn("name")?.getIsVisible())}</output>
      <DataTablePagination table={table} showSelectedRowCount />
    </>
  );
}
it("shared components retain stable selection, column visibility and reset page size", async () => {
  render(<SharedTable />);
  fireEvent.click(screen.getByRole("button", { name: "Select row" }));
  expect(screen.getByText('{"stable-id":true}')).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "Go to next page" }));
  expect(screen.getByText("Page 2 of 3")).toBeTruthy();
  fireEvent.keyDown(screen.getByRole("combobox"), { key: "ArrowDown" });
  const option = await screen.findByRole("option", { name: "20" });
  fireEvent.keyDown(option, { key: "Enter" });
  await waitFor(() => expect(screen.getByText("Page 1 of 3")).toBeTruthy());
  openMenu(screen.getByRole("button", { name: "Name" }));
  fireEvent.click(await screen.findByRole("menuitem", { name: "Hide" }));
  expect(screen.getByText("false")).toBeTruthy();
});

it("formats legacy GraphQL Cron times through the shared boundary without crashing on bad data", async () => {
  await mount(TaskCronManageRouteComponent, "cron");
  expect(await screen.findByText("Invalid timestamp")).toBeTruthy();
  expect(screen.queryByText("Something went wrong")).toBeNull();
  expect(screen.getAllByText(/2026/).length).toBeGreaterThanOrEqual(2);
});
