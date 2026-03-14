import { createFileRoute } from "@tanstack/react-router";
import type { RouteStateDataOption } from "@/infra/routes/traits";

export const Route = createFileRoute("/_app/bangumi/manage")({
  component: BangumiManageRouteComponent,
  staticData: {
    breadcrumb: { label: "Manage" },
  } satisfies RouteStateDataOption,
});

function BangumiManageRouteComponent() {
  return <div>Hello "/_app/bangumi/manage"!</div>;
}
