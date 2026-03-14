import type { Provider } from "@outposts/injection-js";
import { MikanService } from "./services/mikan.service";
import { SubscriptionService } from "./services/subscription.service";

export function provideRecorder(): Provider[] {
  return [
    {
      provide: MikanService,
      useClass: MikanService,
    },
    {
      provide: SubscriptionService,
      useClass: SubscriptionService,
    },
  ];
}
