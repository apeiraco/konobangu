import type { ValueOf } from "type-fest";

export const AUTH_METHOD = {
  BASIC: "basic",
  OIDC: "oidc",
} as const;

export type AuthMethodType = ValueOf<typeof AUTH_METHOD>;

export function getAppAuthMethod(): AuthMethodType {
  return process.env.AUTH__AUTH_TYPE as AuthMethodType;
}
