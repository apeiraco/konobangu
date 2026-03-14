import type { SyncOneSubscriptionFeedsFullTask } from "./SyncOneSubscriptionFeedsFullTask";
import type { SyncOneSubscriptionFeedsIncrementalTask } from "./SyncOneSubscriptionFeedsIncrementalTask";
import type { SyncOneSubscriptionSourcesTask } from "./SyncOneSubscriptionSourcesTask";
export type SubscriberTaskType =
  | ({
      taskType: "sync_one_subscription_feeds_incremental";
    } & SyncOneSubscriptionFeedsIncrementalTask)
  | ({
      taskType: "sync_one_subscription_feeds_full";
    } & SyncOneSubscriptionFeedsFullTask)
  | ({
      taskType: "sync_one_subscription_sources";
    } & SyncOneSubscriptionSourcesTask);
//# sourceMappingURL=SubscriberTaskType.d.ts.map
