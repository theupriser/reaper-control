import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "perf",
  testMatch: "*.perf.ts",
  workers: 1,
  reporter: [["list"]],
  outputDir: "perf-results",
  use: { baseURL: "http://127.0.0.1:4173", trace: "on", viewport: { width: 1280, height: 800 } },
  webServer: { command: "pnpm build && pnpm exec vite preview --host 127.0.0.1 --port 4173 --strictPort", url: "http://127.0.0.1:4173", reuseExistingServer: true, timeout: 120_000 },
});
