import { createFileRoute } from "@tanstack/react-router";
import { AppNotFoundComponent } from "@/components/layout/app-not-found";

export const Route = createFileRoute("/404")({
  component: AppNotFoundComponent,
});
