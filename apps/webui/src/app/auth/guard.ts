import type { ParsedLocation } from "@tanstack/react-router";
import {
  type RouterContext,
  requireRouterInjector,
} from "@/infra/routes/traits";
import { authContextFromInjector } from "./context";

export const beforeLoadGuard = async ({
  context,
  location,
}: {
  context: RouterContext;
  location: ParsedLocation;
}) => {
  const { authService } = authContextFromInjector(
    requireRouterInjector(context),
  );
  await authService.session.start();
  const check = authService.check.get();
  if (check.status === "error") throw new Error(check.message);
  if (!authService.isAuthenticated.get()) {
    await authService.login(location.href);
    throw new Error("Authentication redirect started.");
  }
};
