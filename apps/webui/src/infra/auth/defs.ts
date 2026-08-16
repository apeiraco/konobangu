import type { ValueOf } from "type-fest";

export const AUTH_METHOD = {
  BASIC: "basic",
  OIDC: "oidc",
} as const;

export type AuthMethodType = ValueOf<typeof AUTH_METHOD>;

export function getAppAuthMethod(): AuthMethodType {
  const method = process.env.AUTH__PROVIDER__TYPE;
  if (method !== AUTH_METHOD.BASIC && method !== AUTH_METHOD.OIDC)
    throw new Error("AUTH__PROVIDER__TYPE must be explicitly basic or oidc.");
  return method;
}
