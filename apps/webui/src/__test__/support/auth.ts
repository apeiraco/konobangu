import type {
  BaseTransportTrait,
  HttpRequest,
  HttpResponse,
} from "@securitydept/client";
import { vi } from "vitest";
import { provideAuth } from "@/app/auth/context";
import { AppRuntime } from "@/app/runtime";
import { AuthService } from "@/domains/auth/auth.service";
import { provideRecorder } from "@/domains/recorder/context";
import { provideGraphql } from "@/infra/graphql/context";
import { provideIntl } from "@/infra/intl/context";
import { providePlatform } from "@/infra/platform/context";
import { provideStorages } from "@/infra/storage/context";
import { provideStyles } from "@/infra/styles/context";

export function sessionPayload(subject = "A", issuer = "https://idp.example/") {
  return { principal: { subject, issuer, display_name: subject } };
}

export function createSessionScope(
  handler: (request: HttpRequest) => HttpResponse | Promise<HttpResponse>,
) {
  const execute = vi.fn(handler);
  const transport: BaseTransportTrait = {
    execute: async (request) => execute(request),
  };
  const runtime = new AppRuntime(
    [
      ...providePlatform(),
      ...provideStorages(),
      ...provideStyles(),
      ...provideIntl(),
      ...provideAuth(undefined, "http://localhost"),
      ...provideGraphql(),
      ...provideRecorder(),
    ],
    { transport },
  );
  return { runtime, auth: runtime.injector.get(AuthService), execute };
}

export function response(status: number, body: unknown = null): HttpResponse {
  return { status, headers: {}, body };
}
