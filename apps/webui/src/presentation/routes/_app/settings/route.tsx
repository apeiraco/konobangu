import { createFileRoute } from "@tanstack/react-router";
import { buildVirtualBranchRouteOptions } from "@/infra/routes/utils";

export const Route = createFileRoute("/_app/settings")(
  buildVirtualBranchRouteOptions({
    title: "Settings",
  }),
);
