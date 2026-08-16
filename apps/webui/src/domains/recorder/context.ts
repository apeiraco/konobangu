import type { SecuritydeptProvider as Provider } from "@securitydept/client";
import { Credential3rdService } from "./services/credential3rd.service";
import { MikanService } from "./services/mikan.service";
import { SubscriptionService } from "./services/subscription.service";

export function provideRecorder(): Provider[] {
  return [
    {
      provide: Credential3rdService,
      useFactory: () => new Credential3rdService(),
      deps: [],
    },
    {
      provide: MikanService,
      useFactory: () => new MikanService(),
      deps: [],
    },
    {
      provide: SubscriptionService,
      useFactory: () => new SubscriptionService(),
      deps: [],
    },
  ];
}
