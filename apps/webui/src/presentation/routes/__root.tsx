import { createRootRouteWithContext, Outlet } from "@tanstack/react-router";
import { Home } from "lucide-react";
import { memo } from "react";
import { Toaster } from "sonner";
import type {
  RouterContext,
  RouteStateDataOption,
} from "@/infra/routes/traits";

export const RootRouteComponent = memo(() => {
  return (
    <>
      <Outlet />
      <Toaster position="top-right" />
    </>
  );
});

export const Route = createRootRouteWithContext<RouterContext>()({
  component: RootRouteComponent,
  staticData: {
    breadcrumb: {
      icon: Home,
    },
  } satisfies RouteStateDataOption,
});
