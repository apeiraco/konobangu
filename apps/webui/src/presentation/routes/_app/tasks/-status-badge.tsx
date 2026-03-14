import { AlertCircle, CheckCircle, Clock, Loader2 } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { SubscriberTaskStatusEnum } from "@/infra/graphql/gql/graphql";

export function getStatusBadge(status: SubscriberTaskStatusEnum) {
  switch (status) {
    case SubscriberTaskStatusEnum.Done:
      return (
        <Badge variant="secondary" className="bg-green-100 text-green-800">
          <CheckCircle className="mr-1 h-3 w-3 capitalize" />
          {status}
        </Badge>
      );
    case SubscriberTaskStatusEnum.Running:
      return (
        <Badge variant="secondary" className="bg-blue-100 text-blue-800">
          <Loader2 className="mr-1 h-3 w-3 animate-spin capitalize" />
          {status}
        </Badge>
      );
    case SubscriberTaskStatusEnum.Killed:
    case SubscriberTaskStatusEnum.Failed:
      return (
        <Badge variant="destructive">
          <AlertCircle className="mr-1 h-3 w-3 capitalize" />
          {status}
        </Badge>
      );
    case SubscriberTaskStatusEnum.Scheduled:
    case SubscriberTaskStatusEnum.Pending:
      return (
        <Badge variant="secondary" className="bg-yellow-100 text-yellow-800">
          <Clock className="mr-1 h-3 w-3 capitalize" />
          {status}
        </Badge>
      );
    default:
      return (
        <Badge variant="outline" className="capitalize">
          {status}
        </Badge>
      );
  }
}
