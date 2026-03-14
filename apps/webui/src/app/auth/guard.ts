import type { ParsedLocation } from "@tanstack/react-router";
import { firstValueFrom } from "rxjs";
import type { RouterContext } from "@/infra/routes/traits";
import { authContextFromInjector } from "./context";

export const beforeLoadGuard = async ({
  context,
  location,
}: {
  context: RouterContext;
  location: ParsedLocation;
}) => {
  const { isAuthenticated$, authProvider } = authContextFromInjector(
    context.injector,
  );
  if (!(await firstValueFrom(isAuthenticated$))) {
    const isAuthenticated = await firstValueFrom(
      authProvider.autoLoginPartialRoutesGuard({
        location,
      }),
    );
    if (!isAuthenticated) {
      throw !isAuthenticated;
    }
  }
};
