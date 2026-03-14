import type { Injector, Provider } from "@outposts/injection-js";
import { IntlService } from "./intl.service";

export function provideIntl(): Provider[] {
  return [IntlService];
}

export function intlContextFromInjector(injector: Injector) {
  const intlService = injector.get(IntlService);

  return {
    intlService,
  };
}
