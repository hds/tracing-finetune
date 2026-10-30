import { defineConfig } from '@playwright/test';

export default defineConfig({
    testDir: './tests',
    use: { 
        baseURL: 'http://localhost:3007',
        trace: 'on',
    },
    webServer: [
        {
            command: 'cargo run --features axum-router --example example-server-axum',
            url: 'http://localhost:3007/health',
            reuseExistingServer: !process.env.CI,
            timeout: 120_000,
        }
    ],
});
