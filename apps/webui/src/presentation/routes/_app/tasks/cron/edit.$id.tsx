import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/_app/tasks/cron/edit/$id")({
	component: RouteComponent,
});

function RouteComponent() {
	return <div>Hello "/_app/tasks/cron/edit/$id"!</div>;
}
