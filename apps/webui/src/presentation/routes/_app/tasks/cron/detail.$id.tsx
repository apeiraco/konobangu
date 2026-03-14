import { useQuery } from "@apollo/client/react";
import { createFileRoute } from "@tanstack/react-router";
import { RefreshCw } from "lucide-react";
import { useMemo } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { ContainerHeader } from "@/components/ui/container-header";
import { CronDisplay } from "@/components/ui/cron";
import { DetailCardSkeleton } from "@/components/ui/detail-card-skeleton";
import { DetailEmptyView } from "@/components/ui/detail-empty-view";
import { Label } from "@/components/ui/label";
import { QueryErrorView } from "@/components/ui/query-error-view";
import { Separator } from "@/components/ui/separator";
import { GET_CRONS } from "@/domains/recorder/schema/cron";
import { useInject } from "@/infra/di/inject";
import {
  CronStatusEnum,
  type GetCronsQuery,
  type GetCronsQueryVariables,
} from "@/infra/graphql/gql/graphql";
import { IntlService } from "@/infra/intl/intl.service";
import type { RouteStateDataOption } from "@/infra/routes/traits";
import { getStatusBadge } from "./-status-badge";

export const Route = createFileRoute("/_app/tasks/cron/detail/$id")({
  component: CronDetailRouteComponent,
  staticData: {
    breadcrumb: { label: "Detail" },
  } satisfies RouteStateDataOption,
});

function CronDetailRouteComponent() {
  const { id } = Route.useParams();
  const intlService = useInject(IntlService);

  const { data, loading, error, refetch } = useQuery<
    GetCronsQuery,
    GetCronsQueryVariables
  >(GET_CRONS, {
    variables: {
      filter: {
        id: {
          eq: Number.parseInt(id, 10),
        },
      },
      pagination: {
        page: {
          page: 0,
          limit: 1,
        },
      },
      orderBy: {},
    },
    pollInterval: 5000, // Auto-refresh every 5 seconds for running crons
  });

  const cron = data?.cron?.nodes?.[0];

  const subscriberTaskCron = useMemo(() => {
    if (!cron) {
      return null;
    }
    return cron.subscriberTaskCron;
  }, [cron]);

  if (loading) {
    return <DetailCardSkeleton />;
  }

  if (error) {
    return <QueryErrorView message={error.message} onRetry={refetch} />;
  }

  if (!cron) {
    return <DetailEmptyView message="Not found Cron task" />;
  }

  return (
    <div className="container mx-auto max-w-4xl py-6">
      <ContainerHeader
        title="Cron task detail"
        description={`View Cron task #${cron.id}`}
        defaultBackTo="/tasks/cron/manage"
        actions={
          <Button variant="outline" size="sm" onClick={() => refetch()}>
            <RefreshCw className="h-4 w-4" />
            Refresh
          </Button>
        }
      />

      <div className="space-y-6">
        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <div>
                <CardTitle>Cron task information</CardTitle>
                <CardDescription className="mt-2">
                  View Cron task execution details
                </CardDescription>
              </div>
              <div className="flex items-center gap-2">
                {getStatusBadge(cron.status)}
              </div>
            </div>
          </CardHeader>
          <CardContent>
            <div className="space-y-6">
              {/* Basic Information */}
              <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
                <div className="space-y-2">
                  <Label className="font-medium text-sm">ID</Label>
                  <div className="rounded-md bg-muted p-3">
                    <code className="text-sm">{cron.id}</code>
                  </div>
                </div>

                <div className="space-y-2">
                  <Label className="font-medium text-sm">Priority</Label>
                  <div className="rounded-md bg-muted p-3">
                    <span className="text-sm">{cron.priority}</span>
                  </div>
                </div>

                <div className="space-y-2">
                  <Label className="font-medium text-sm">Attemps</Label>
                  <div className="rounded-md bg-muted p-3">
                    <span className="text-sm">
                      {cron.attempts} / {cron.maxAttempts}
                    </span>
                  </div>
                </div>

                <div className="space-y-2">
                  <Label className="font-medium text-sm">Enabled</Label>
                  <div className="rounded-md bg-muted p-3">
                    <Badge variant={cron.enabled ? "default" : "secondary"}>
                      {cron.enabled ? "Enabled" : "Disabled"}
                    </Badge>
                  </div>
                </div>

                <div className="space-y-2">
                  <Label className="font-medium text-sm">Next run time</Label>
                  <div className="rounded-md bg-muted p-3">
                    <span className="text-sm">
                      {cron.nextRun
                        ? intlService.formatDatetimeWithTz(cron.nextRun)
                        : "-"}
                    </span>
                  </div>
                </div>

                <div className="space-y-2">
                  <Label className="font-medium text-sm">Last run time</Label>
                  <div className="rounded-md bg-muted p-3">
                    <span className="text-sm">
                      {cron.lastRun
                        ? intlService.formatDatetimeWithTz(cron.lastRun)
                        : "-"}
                    </span>
                  </div>
                </div>

                <div className="space-y-2">
                  <Label className="font-medium text-sm">Locked time</Label>
                  <div className="rounded-md bg-muted p-3">
                    <span className="text-sm">
                      {cron.lockedAt
                        ? intlService.formatDatetimeWithTz(cron.lockedAt)
                        : "-"}
                    </span>
                  </div>
                </div>

                <div className="space-y-2">
                  <Label className="font-medium text-sm">Locked by</Label>
                  <div className="rounded-md bg-muted p-3">
                    <code className="text-sm">{cron.lockedBy || "-"}</code>
                  </div>
                </div>

                <div className="space-y-2">
                  <Label className="font-medium text-sm">Timeout</Label>
                  <div className="rounded-md bg-muted p-3">
                    <span className="text-sm">
                      {cron.timeoutMs ? `${cron.timeoutMs}ms` : "No limit"}
                    </span>
                  </div>
                </div>

                <div className="space-y-2">
                  <Label className="font-medium text-sm">Created at</Label>
                  <div className="rounded-md bg-muted p-3">
                    <span className="text-sm">
                      {intlService.formatDatetimeWithTz(cron.createdAt)}
                    </span>
                  </div>
                </div>

                <div className="space-y-2">
                  <Label className="font-medium text-sm">Updated at</Label>
                  <div className="rounded-md bg-muted p-3">
                    <span className="text-sm">
                      {intlService.formatDatetimeWithTz(cron.updatedAt)}
                    </span>
                  </div>
                </div>
              </div>

              {/* Subscriber Task Details */}
              {subscriberTaskCron && (
                <>
                  <Separator />
                  <div className="space-y-2">
                    <Label className="font-medium text-sm">
                      Subscriber task details
                    </Label>
                    <div className="rounded-md bg-muted p-3">
                      <pre className="overflow-x-auto whitespace-pre-wrap text-sm">
                        <code>
                          {JSON.stringify(subscriberTaskCron, null, 2)}
                        </code>
                      </pre>
                    </div>
                  </div>
                </>
              )}

              {/* Cron Expression Display */}
              {cron.cronExpr && (
                <>
                  <Separator />
                  <div className="space-y-2">
                    <Label className="font-medium text-sm">
                      Cron expression
                    </Label>
                    <CronDisplay
                      expression={cron.cronExpr}
                      timezone="UTC"
                      showDescription={true}
                      showNextRuns={true}
                      withCard={false}
                    />
                  </div>
                </>
              )}

              {/* Related Subscriber Tasks */}
              {cron.subscriberTask?.nodes &&
                cron.subscriberTask.nodes.length > 0 && (
                  <>
                    <Separator />
                    <div className="space-y-2">
                      <Label className="font-medium text-sm">
                        Associated tasks
                      </Label>
                      <div className="space-y-2">
                        {cron.subscriberTask.nodes.map((task, index) => (
                          <div
                            key={`${task.id}-${index}`}
                            className="rounded-md border bg-muted/50 p-3"
                          >
                            <div className="flex items-center justify-between">
                              <code className="text-sm">{task.id}</code>
                              <Badge variant="outline">{task.status}</Badge>
                            </div>
                            <div className="mt-2 text-muted-foreground text-sm">
                              Priority: {task.priority} | Attempts:{" "}
                              {task.attempts}/{task.maxAttempts}
                            </div>
                            {task.subscription && (
                              <div className="mt-2 text-sm">
                                <span className="font-medium">
                                  Subscription:
                                </span>{" "}
                                {task.subscription.displayName}
                              </div>
                            )}
                          </div>
                        ))}
                      </div>
                    </div>
                  </>
                )}

              {/* Error Information */}
              {cron.status === CronStatusEnum.Failed && cron.lastError && (
                <>
                  <Separator />
                  <div className="space-y-2">
                    <Label className="font-medium text-sm">最后错误</Label>
                    <div className="rounded-md bg-destructive/10 p-3">
                      <p className="text-destructive text-sm">
                        {cron.lastError}
                      </p>
                    </div>
                  </div>
                </>
              )}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
