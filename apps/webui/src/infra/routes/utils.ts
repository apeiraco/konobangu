import { Outlet, type RouteOptions } from "@tanstack/react-router";
import { guardRouteIndexAsNotFound } from "@/components/layout/app-not-found";
import type { RouteStateDataOption } from "@/infra/routes/traits";

export interface BuildVirtualBranchRouteOptions {
	title: string;
}

export function buildVirtualBranchRouteOptions(
	options: BuildVirtualBranchRouteOptions,
): {
	beforeLoad: RouteOptions["beforeLoad"];
	staticData: RouteStateDataOption;
	component: RouteOptions["component"];
} {
	return {
		beforeLoad: guardRouteIndexAsNotFound,
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
