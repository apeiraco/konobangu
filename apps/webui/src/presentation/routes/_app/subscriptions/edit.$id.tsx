import { useMutation, useQuery } from "@apollo/client/react";
import { createFileRoute } from "@tanstack/react-router";
import { Save } from "lucide-react";
import { useCallback, useMemo } from "react";
import { toast } from "sonner";
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
import { DetailCardSkeleton } from "@/components/ui/detail-card-skeleton";
import { DetailEmptyView } from "@/components/ui/detail-empty-view";
import { FormFieldErrors } from "@/components/ui/form-field-errors";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { QueryErrorView } from "@/components/ui/query-error-view";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { useAppForm } from "@/components/ui/tanstack-form";
import { MikanSeasonEnum } from "@/domains/recorder/schema/mikan";
import {
  GET_SUBSCRIPTION_DETAIL,
  type SubscriptionForm,
  SubscriptionFormSchema,
  UPDATE_SUBSCRIPTIONS,
} from "@/domains/recorder/schema/subscriptions";
import { SubscriptionService } from "@/domains/recorder/services/subscription.service";
import { useInject } from "@/infra/di/inject";
import {
  apolloErrorToMessage,
  getApolloQueryError,
} from "@/infra/errors/apollo";
import { compatFormDefaultValues } from "@/infra/forms/compat";
import {
  Credential3rdTypeEnum,
  type GetSubscriptionDetailQuery,
  SubscriptionCategoryEnum,
  type UpdateSubscriptionsMutation,
  type UpdateSubscriptionsMutationVariables,
} from "@/infra/graphql/gql/graphql";
import type { RouteStateDataOption } from "@/infra/routes/traits";
import { Credential3rdSelectContent } from "./-credential3rd-select";

export const Route = createFileRoute("/_app/subscriptions/edit/$id")({
  component: SubscriptionEditRouteComponent,
  staticData: {
    breadcrumb: { label: "Edit" },
  } satisfies RouteStateDataOption,
});

type SubscriptionDetailDto = NonNullable<
  GetSubscriptionDetailQuery["subscriptions"]["nodes"][0]
>;

function FormView({
  subscription,
  onCompleted,
}: {
  subscription: SubscriptionDetailDto;
  onCompleted: VoidFunction;
}) {
  const subscriptionService = useInject(SubscriptionService);

  const [updateSubscription, { loading: updating }] = useMutation<
    UpdateSubscriptionsMutation,
    UpdateSubscriptionsMutationVariables
  >(UPDATE_SUBSCRIPTIONS, {
    onCompleted,
    onError: (error) => {
      toast.error("Update subscription failed", {
        description: error.message,
      });
    },
  });

  // Extract source URL metadata for form initialization
  const sourceUrlMeta = useMemo(
    () =>
      subscriptionService.extractSourceUrlMeta(
        subscription.category,
        subscription.sourceUrl,
      ),
    [subscription.category, subscription.sourceUrl, subscriptionService],
  );

  // Initialize form with current subscription data
  const defaultValues = useMemo(() => {
    const base = {
      displayName: subscription.displayName,
      category: subscription.category,
      enabled: subscription.enabled,
      sourceUrl: subscription.sourceUrl,
      credentialId: subscription.credential3rd?.id ?? Number.NaN,
      year: Number.NaN,
      seasonStr: "",
    };

    if (
      subscription.category === SubscriptionCategoryEnum.MikanSeason &&
      sourceUrlMeta?.category === SubscriptionCategoryEnum.MikanSeason
    ) {
      return {
        ...base,
        year: sourceUrlMeta.year,
        seasonStr: sourceUrlMeta.seasonStr,
      };
    }

    return base;
  }, [subscription, sourceUrlMeta]);

  const form = useAppForm({
    defaultValues: compatFormDefaultValues<SubscriptionForm>(defaultValues),
    validators: {
      onChangeAsync: SubscriptionFormSchema,
      onChangeAsyncDebounceMs: 300,
      onSubmit: SubscriptionFormSchema,
    },
    onSubmit: async (submittedForm) => {
      const input = subscriptionService.transformInsertFormToInput(
        submittedForm.value,
      );

      await updateSubscription({
        variables: {
          data: input,
          filter: {
            id: {
              eq: subscription.id,
            },
          },
        },
      });
    },
  });

  return (
    <div className="container mx-auto max-w-4xl py-6">
      <ContainerHeader
        title="Subscription Edit"
        description={`Edit subscription #${subscription.id}`}
        defaultBackTo={`/subscriptions/detail/${subscription.id}`}
        actions={
          <Button onClick={() => form.handleSubmit()} disabled={updating}>
            <Save className="mr-2 h-4 w-4" />
            {updating ? "Saving..." : "Save"}
          </Button>
        }
      />

      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>Subscription information</CardTitle>
              <CardDescription className="mt-2">
                Edit subscription information
              </CardDescription>
            </div>
            <Badge variant="outline">{subscription.category}</Badge>
          </div>
        </CardHeader>
        <CardContent>
          <form
            onSubmit={(e) => {
              e.preventDefault();
              e.stopPropagation();
              form.handleSubmit();
            }}
            className="space-y-6"
          >
            <form.Field name="displayName">
              {(field) => (
                <div className="space-y-2">
                  <Label htmlFor={field.name}>Display Name *</Label>
                  <Input
                    id={field.name}
                    name={field.name}
                    value={field.state.value}
                    onBlur={field.handleBlur}
                    onChange={(e) => field.handleChange(e.target.value)}
                    placeholder="Please enter display name"
                    autoComplete="off"
                  />
                  {field.state.meta.errors && (
                    <FormFieldErrors
                      errors={field.state.meta.errors}
                      isDirty={field.state.meta.isDirty}
                      submissionAttempts={form.state.submissionAttempts}
                    />
                  )}
                </div>
              )}
            </form.Field>

            {/* Category is read-only in edit mode */}
            <div className="space-y-2">
              <Label>Category</Label>
              <div className="rounded-md bg-muted p-3">
                <Badge variant="outline">{subscription.category}</Badge>
              </div>
            </div>

            {/* Conditional fields based on category */}
            {subscription.category === SubscriptionCategoryEnum.MikanSeason ? (
              <>
                <form.Field name="credentialId">
                  {(field) => (
                    <div className="space-y-2">
                      <Label htmlFor={field.name}>Credential ID *</Label>
                      <Select
                        value={field.state.value.toString()}
                        onValueChange={(value) =>
                          field.handleChange(Number.parseInt(value, 10))
                        }
                      >
                        <SelectTrigger>
                          <SelectValue placeholder="Select credential" />
                        </SelectTrigger>
                        <Credential3rdSelectContent
                          credentialType={Credential3rdTypeEnum.Mikan}
                        />
                      </Select>
                      {field.state.meta.errors && (
                        <FormFieldErrors
                          errors={field.state.meta.errors}
                          isDirty={field.state.meta.isDirty}
                          submissionAttempts={form.state.submissionAttempts}
                        />
                      )}
                    </div>
                  )}
                </form.Field>
                <form.Field name="year">
                  {(field) => (
                    <div className="space-y-2">
                      <Label htmlFor={field.name}>Year *</Label>
                      <Input
                        id={field.name}
                        name={field.name}
                        value={field.state.value}
                        type="number"
                        min={1970}
                        onBlur={field.handleBlur}
                        onChange={(e) =>
                          field.handleChange(
                            Number.parseInt(e.target.value, 10),
                          )
                        }
                        placeholder={`Please enter full year (e.g. ${new Date().getFullYear()})`}
                        autoComplete="off"
                      />
                      {field.state.meta.errors && (
                        <FormFieldErrors
                          errors={field.state.meta.errors}
                          isDirty={field.state.meta.isDirty}
                          submissionAttempts={form.state.submissionAttempts}
                        />
                      )}
                    </div>
                  )}
                </form.Field>
                <form.Field name="seasonStr">
                  {(field) => (
                    <div className="space-y-2">
                      <Label htmlFor={field.name}>Season *</Label>
                      <Select
                        value={field.state.value}
                        onValueChange={(value) =>
                          field.handleChange(value as MikanSeasonEnum)
                        }
                      >
                        <SelectTrigger>
                          <SelectValue placeholder="Select season" />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value={MikanSeasonEnum.Spring}>
                            Spring
                          </SelectItem>
                          <SelectItem value={MikanSeasonEnum.Summer}>
                            Summer
                          </SelectItem>
                          <SelectItem value={MikanSeasonEnum.Autumn}>
                            Autumn
                          </SelectItem>
                          <SelectItem value={MikanSeasonEnum.Winter}>
                            Winter
                          </SelectItem>
                        </SelectContent>
                      </Select>
                      {field.state.meta.errors && (
                        <FormFieldErrors
                          errors={field.state.meta.errors}
                          isDirty={field.state.meta.isDirty}
                          submissionAttempts={form.state.submissionAttempts}
                        />
                      )}
                    </div>
                  )}
                </form.Field>
              </>
            ) : (
              <form.Field name="sourceUrl">
                {(field) => (
                  <div className="space-y-2">
                    <Label htmlFor={field.name}>Source URL *</Label>
                    <Input
                      id={field.name}
                      name={field.name}
                      value={field.state.value}
                      onBlur={field.handleBlur}
                      onChange={(e) => field.handleChange(e.target.value)}
                      placeholder="Please enter source URL"
                      autoComplete="off"
                    />
                    {field.state.meta.errors && (
                      <FormFieldErrors
                        errors={field.state.meta.errors}
                        isDirty={field.state.meta.isDirty}
                        submissionAttempts={form.state.submissionAttempts}
                      />
                    )}
                  </div>
                )}
              </form.Field>
            )}

            <form.Field name="enabled">
              {(field) => (
                <div className="flex items-center justify-between">
                  <div className="space-y-0.5">
                    <Label htmlFor={field.name}>Enabled</Label>
                    <div className="text-muted-foreground text-sm">
                      Enable this subscription
                    </div>
                  </div>
                  <Switch
                    id={field.name}
                    checked={field.state.value}
                    onCheckedChange={(checked) => field.handleChange(checked)}
                  />
                </div>
              )}
            </form.Field>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}

function SubscriptionEditRouteComponent() {
  const { id } = Route.useParams();

  const { loading, error, data, refetch } =
    useQuery<GetSubscriptionDetailQuery>(GET_SUBSCRIPTION_DETAIL, {
      variables: {
        id: Number.parseInt(id, 10),
      },
    });

  const subscription = data?.subscriptions?.nodes?.[0];

  const onCompleted = useCallback(async () => {
    const refetchResult = await refetch();
    const _error = getApolloQueryError(refetchResult);
    if (_error) {
      toast.error("Update subscription failed", {
        description: apolloErrorToMessage(_error),
      });
    } else {
      toast.success("Update subscription successfully");
    }
  }, [refetch]);

  if (loading) {
    return <DetailCardSkeleton />;
  }

  if (error) {
    return <QueryErrorView message={error.message} />;
  }

  if (!subscription) {
    return <DetailEmptyView message="Not found certain subscription" />;
  }

  return <FormView subscription={subscription} onCompleted={onCompleted} />;
}
