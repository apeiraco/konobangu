import { createFileRoute } from "@tanstack/react-router";
import { buildVirtualBranchRouteOptions } from "@/infra/routes/utils";

export const Route = createFileRoute("/_app/credential3rd")(
  buildVirtualBranchRouteOptions({
    path: "/credential3rd",
    title: "Credential",
  }),
);
