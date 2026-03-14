/// <reference types="vite/client" />

// Type declarations for variables injected by Vite's `define` option.
// These are compile-time string replacements, NOT actual Node.js env vars.
declare namespace NodeJS {
  interface ProcessEnv {
    AUTH__AUTH_TYPE?: string;
    AUTH__OIDC_CLIENT_ID?: string;
    AUTH__OIDC_CLIENT_SECRET?: string;
    AUTH__OIDC_ISSUER?: string;
    AUTH__OIDC_AUDIENCE?: string;
    AUTH__OIDC_EXTRA_SCOPES?: string;
  }
}

declare const process: {
  env: NodeJS.ProcessEnv;
};
