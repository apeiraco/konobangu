import { defineConfig } from "@playwright/test";

const files = "**/*.{spec,test}.{ts,tsx,mts,cts,js,jsx,mjs,cjs}";

export default defineConfig({
  testDir: "./tests",
  fullyParallel: true,
  retries: 0,
  timeout: 30000,
  outputDir: "../../temp/verification/browser/playwright-results",
  reporter: "list",
  use: { browserName: "chromium", trace: "retain-on-failure" },
  projects: [
    { name: "integration", testMatch: `**/integration/${files}` },
    { name: "e2e", testMatch: `**/e2e/${files}` },
  ],
});
