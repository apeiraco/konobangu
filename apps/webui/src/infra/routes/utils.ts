import { Outlet, type ParsedLocation } from "@tanstack/react-router";
import { guardRouteIndexAsNotFound } from "@/components/layout/app-not-found";
import type { RouteStateDataOption } from "@/infra/routes/traits";

export interface BuildVirtualBranchRouteOptions {
  title: string;
  path: string;
}

export function buildVirtualBranchRouteOptions(
  options: BuildVirtualBranchRouteOptions,
) {
  return {
    beforeLoad: (context: { location: ParsedLocation }) =>
      guardRouteIndexAsNotFound(options.path, context),
    staticData: {
      breadcrumb: {
        label: options.title,
        link: undefined,
      },
    } satisfies RouteStateDataOption,
    component: Outlet,
  };
}

export interface BuildLeafRouteStaticDataOptions {
  title: string;
}

export function buildLeafRouteStaticData(
  options: BuildLeafRouteStaticDataOptions,
): RouteStateDataOption {
  return {
    breadcrumb: {
      label: options.title,
    },
  };
}
