import { createFileRoute, Outlet } from "@tanstack/react-router";
import { beforeLoadGuard } from "@/app/auth/guard";
import { AppAside } from "@/components/layout/app-layout";

export const Route = createFileRoute("/_app")({
	component: AppLayoutRoute,
	beforeLoad: beforeLoadGuard,
});

function AppLayoutRoute() {
	return (
		<AppAside extractBreadcrumbFromRoutes>
			<Outlet />
		</AppAside>
	);
}
