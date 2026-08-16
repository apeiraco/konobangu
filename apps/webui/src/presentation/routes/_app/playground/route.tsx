import { createFileRoute } from "@tanstack/react-router";
import { buildVirtualBranchRouteOptions } from "@/infra/routes/utils";

export const Route = createFileRoute("/_app/playground")(
  buildVirtualBranchRouteOptions({
    path: "/playground",
    title: "Playground",
  }),
);
