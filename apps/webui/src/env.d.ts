/// <reference types="vite/client" />

// Type declarations for variables injected by Vite's `define` option.
// These are compile-time string replacements, NOT actual Node.js env vars.
declare namespace NodeJS {
  interface ProcessEnv {
    AUTH__PROVIDER__TYPE?: string;
  }
}

declare const process: {
  env: NodeJS.ProcessEnv;
};
