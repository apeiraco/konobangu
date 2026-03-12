import { createFileRoute } from "@tanstack/react-router";
import { buildVirtualBranchRouteOptions } from "@/infra/routes/utils";

export const Route = createFileRoute("/_app/tasks/cron")(
	buildVirtualBranchRouteOptions({
		title: "Cron",
	}),
);
