import { useMutation } from "@apollo/client/react";
import {
  createFileRoute,
  useCanGoBack,
  useNavigate,
  useRouter,
} from "@tanstack/react-router";
import { type } from "arktype";
import { Loader2, Save } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { ContainerHeader } from "@/components/ui/container-header";
import { FormFieldErrors } from "@/components/ui/form-field-errors";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useAppForm } from "@/components/ui/tanstack-form";
import { Textarea } from "@/components/ui/textarea";
import {
  Credential3rdInsertSchema,
  INSERT_CREDENTIAL_3RD,
} from "@/domains/recorder/schema/credential3rd";
import { useInject } from "@/infra/di/inject";
import { compatFormDefaultValues } from "@/infra/forms/compat";
import {
  type Credential3rdInsertInput,
  Credential3rdTypeEnum,
  type InsertCredential3rdMutation,
  type InsertCredential3rdMutationVariables,
} from "@/infra/graphql/gql/graphql";
import { PlatformService } from "@/infra/platform/platform.service";
import {
  CreateCompleteAction,
  CreateCompleteActionSchema,
} from "@/infra/routes/nav";
import type { RouteStateDataOption } from "@/infra/routes/traits";

const RouteSearchSchema = type({
  completeAction: CreateCompleteActionSchema.optional(),
});

export const Route = createFileRoute("/_app/credential3rd/create")({
  component: CredentialCreateRouteComponent,
  staticData: {
    breadcrumb: { label: "Create" },
  } satisfies RouteStateDataOption,
  validateSearch: RouteSearchSchema,
});

function CredentialCreateRouteComponent() {
  const navigate = useNavigate();
  const router = useRouter();
  const canGoBack = useCanGoBack();
  const search = Route.useSearch();
  const platformService = useInject(PlatformService);

  const handleBack = () => {
    if (canGoBack) {
      router.history.back();
    } else {
      navigate({
        to: "/credential3rd/manage",
      });
    }
  };

  const [insertCredential3rd, { loading }] = useMutation<
    InsertCredential3rdMutation,
    InsertCredential3rdMutationVariables
  >(INSERT_CREDENTIAL_3RD, {
    onCompleted(data) {
      toast.success("Credential created");
      if (search.completeAction === CreateCompleteAction.Back) {
        handleBack();
      } else {
        navigate({
          to: "/credential3rd/detail/$id",
          params: { id: `${data.credential3rdCreateOne.id}` },
        });
      }
    },
    onError(error) {
      toast.error("Failed to create credential", {
        description: error.message,
      });
    },
  });

  const form = useAppForm({
    defaultValues: compatFormDefaultValues<
      Credential3rdInsertInput,
      "credentialType" | "username" | "password" | "userAgent"
    >({
      credentialType: Credential3rdTypeEnum.Mikan,
      username: "",
      password: "",
      userAgent: "",
    }),
    validators: {
      onChangeAsync: Credential3rdInsertSchema,
      onChangeAsyncDebounceMs: 300,
      onSubmit: Credential3rdInsertSchema,
    },
    onSubmit: async (submittedForm) => {
      const value = {
        ...submittedForm.value,
        userAgent: submittedForm.value.userAgent || platformService.userAgent,
      };
      await insertCredential3rd({
        variables: {
          data: value,
        },
      });
    },
  });

  return (
    <div className="container mx-auto max-w-2xl py-6">
      <ContainerHeader
        title="Create third-party credential"
        description="Add new third-party login credential"
        defaultBackTo="/credential3rd/manage"
      />

      <Card>
        <CardHeader>
          <CardTitle>Credential information</CardTitle>
          <CardDescription className="mt-2">
            Please fill in the information of the third-party platform login
            credential for your subscriptions.
          </CardDescription>
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
            <form.Field name="credentialType">
              {(field) => (
                <div className="space-y-2">
                  <Label htmlFor={field.name}>Credential type *</Label>
                  <Select
                    value={field.state.value}
                    onValueChange={(value) =>
                      field.handleChange(value as Credential3rdTypeEnum)
                    }
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select credential type" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value={Credential3rdTypeEnum.Mikan}>
                        Mikan
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
            <form.Field name="username">
              {(field) => (
                <div className="space-y-2">
                  <Label htmlFor={field.name}>Username *</Label>
                  <Input
                    id={field.name}
                    name={field.name}
                    value={field.state.value ?? ""}
                    onBlur={field.handleBlur}
                    onChange={(e) => field.handleChange(e.target.value)}
                    placeholder="Please enter username"
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
            <form.Field name="password">
              {(field) => (
                <div className="space-y-2">
                  <Label htmlFor={field.name}>Password *</Label>
                  <Input
                    id={field.name}
                    name={field.name}
                    type="password"
                    value={field.state.value ?? ""}
                    onBlur={field.handleBlur}
                    onChange={(e) => field.handleChange(e.target.value)}
                    placeholder="Please enter password"
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
            <form.Field name="userAgent">
              {(field) => (
                <div className="space-y-2">
                  <Label htmlFor={field.name}>User Agent</Label>
                  <Textarea
                    id={field.name}
                    name={field.name}
                    value={field.state.value || ""}
                    onBlur={field.handleBlur}
                    onChange={(e) => field.handleChange(e.target.value)}
                    rows={3}
                    className="resize-none"
                    autoComplete="off"
                    placeholder="Please enter User Agent (optional), leave it blank to use
                    the default value"
                  />
                  <p className="text-muted-foreground text-sm">
                    Current default user agent: {platformService.userAgent}
                  </p>
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

            <div className="flex gap-3 pt-4">
              <form.Subscribe
                selector={(state) => [state.canSubmit, state.isSubmitting]}
              >
                {([canSubmit, isSubmitting]) => (
                  <Button
                    type="submit"
                    disabled={!canSubmit || loading}
                    className="flex-1"
                  >
                    {loading || isSubmitting ? (
                      <>
                        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                        Creating...
                      </>
                    ) : (
                      <>
                        <Save className="mr-2 h-4 w-4" />
                        Create credential
                      </>
                    )}
                  </Button>
                )}
              </form.Subscribe>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
