import { atomWithObservable } from "jotai/utils";
import { useInjector } from "oidc-client-rx/adapters/react";
import { useMemo } from "react";
import type { Observable } from "rxjs";
import { authContextFromInjector } from "./context";

export function useAuth() {
  const injector = useInjector();

  const authContext = useMemo(
    () => authContextFromInjector(injector),
    [injector],
  );

  const isAuthenticated = useMemo(
    () =>
      atomWithObservable(
        () => authContext.isAuthenticated$ as Observable<boolean>,
      ),
    [authContext.isAuthenticated$],
  );

  const authData = useMemo(
    () => atomWithObservable(() => authContext.userData$ as Observable<any>),
    [authContext],
  );

  return {
    ...authContext,
    authData,
    injector,
    isAuthenticated,
  };
}
