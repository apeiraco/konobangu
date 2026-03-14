import type { Injector } from "@outposts/injection-js";
import { atomWithObservable } from "jotai/utils";
import { useInjector } from "oidc-client-rx/adapters/react";
import { useMemo } from "react";
import { ThemeService } from "./theme.service";

export function provideStyles() {
  return [
    {
      provide: ThemeService,
      useClass: ThemeService,
    },
  ];
}

export function themeContextFromInjector(injector: Injector) {
  const themeService = injector.get(ThemeService);
  const systemColorSchema$ = atomWithObservable(
    () => themeService.systemColorSchema$,
  );
  return {
    themeService,
    systemColorSchema$,
  };
}

export function setupThemeContext(injector: Injector) {
  const { themeService } = themeContextFromInjector(injector);
  themeService.setup();
}

export function useTheme() {
  const injector = useInjector();

  const { themeService } = useMemo(() => {
    return themeContextFromInjector(injector);
  }, [injector]);

  const colorTheme = useMemo(
    () =>
      atomWithObservable(() => themeService.colorSchema$, {
        initialValue: themeService.colorSchema$.value,
      }),
    [themeService.colorSchema$],
  );

  return {
    themeService,
    colorTheme,
  };
}
