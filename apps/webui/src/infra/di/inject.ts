import { useSecuritydeptContext } from "@securitydept/client-react";
import type { InjectionToken, Type } from "injection-js";
import { useMemo } from "react";

export function useInject<T>(token: InjectionToken<T> | Type<T>): T {
  const injector = useSecuritydeptContext();
  return useMemo(() => injector.get(token), [injector, token]);
}
