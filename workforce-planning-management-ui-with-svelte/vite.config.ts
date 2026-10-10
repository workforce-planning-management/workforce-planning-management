import adapter from "@sveltejs/adapter-auto";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";
import { sveltekit } from "@sveltejs/kit/vite";
import { svelteTesting } from "@testing-library/svelte/vite";
import { defineConfig } from "vitest/config";

export default defineConfig({
    // `svelteTesting()` selects Svelte's browser/client resolve condition so
    // components mount client-side under jsdom (rather than SSR), and wires
    // testing-library auto-cleanup. It only affects the vitest run.
    plugins: [
        sveltekit({
            preprocess: vitePreprocess(),
            adapter: adapter(),
            // Nonces and hashes for SvelteKit's own inline scripts are added
            // automatically; nothing else may run (WPM-R73).
            csp: {
                mode: "auto",
                directives: {
                    "default-src": ["self"],
                    "script-src": ["self"],
                    // Svelte sets inline `style` attributes.
                    "style-src": ["self", "unsafe-inline"],
                    "img-src": ["self", "data:"],
                    "font-src": ["self", "data:"],
                    "connect-src": ["self"],
                    "object-src": ["none"],
                    "base-uri": ["self"],
                    "form-action": ["self"],
                    "frame-ancestors": ["none"]
                }
            }
        }),
        svelteTesting()
    ],
    server: { port: 5173, strictPort: false },
    test: {
        include: ["tests/unit/**/*.{test,spec}.{ts,js}"],
        environment: "jsdom",
        globals: true,
    },
});
