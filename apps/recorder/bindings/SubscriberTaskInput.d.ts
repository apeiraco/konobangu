import type { SyncOneSubscriptionFeedsFullTaskInput } from "./SyncOneSubscriptionFeedsFullTaskInput";
import type { SyncOneSubscriptionFeedsIncrementalTaskInput } from "./SyncOneSubscriptionFeedsIncrementalTaskInput";
import type { SyncOneSubscriptionSourcesTaskInput } from "./SyncOneSubscriptionSourcesTaskInput";
export type SubscriberTaskInput =
  | ({
      taskType: "sync_one_subscription_feeds_incremental";
    } & SyncOneSubscriptionFeedsIncrementalTaskInput)
  | ({
      taskType: "sync_one_subscription_feeds_full";
    } & SyncOneSubscriptionFeedsFullTaskInput)
  | ({
      taskType: "sync_one_subscription_sources";
    } & SyncOneSubscriptionSourcesTaskInput);
//# sourceMappingURL=SubscriberTaskInput.d.ts.map
