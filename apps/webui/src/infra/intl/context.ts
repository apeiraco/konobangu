import type {
  SecuritydeptInjectorTrait as Injector,
  SecuritydeptProvider as Provider,
} from "@securitydept/client";
import { IntlService } from "./intl.service";

export function provideIntl(): Provider[] {
  return [
    { provide: IntlService, useFactory: () => new IntlService(), deps: [] },
  ];
}

export function intlContextFromInjector(injector: Injector) {
  const intlService = injector.get(IntlService);

  return {
    intlService,
  };
}
