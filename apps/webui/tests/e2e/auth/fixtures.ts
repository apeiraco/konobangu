import { test as base } from "@playwright/test";
import { startAuthFixture } from "./run.mts";

type AuthFixture = Awaited<ReturnType<typeof startAuthFixture>>;

// Only suites importing this fixture acquire backend/IdP resources, after discovery.
export const test = base.extend<{}, { authFixture: AuthFixture }>({
  authFixture: [
    // biome-ignore lint/correctness/noEmptyPattern: Playwright requires destructuring to declare fixture dependencies.
    async ({}, use) => {
      await using fixture = await startAuthFixture();
      await use(fixture);
    },
    { scope: "worker", auto: true, timeout: 15 * 60 * 1000 },
  ],
});
