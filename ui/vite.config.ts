import { defineConfig } from "vite"
import solid from "vite-plugin-solid"
import { nodePolyfills } from "vite-plugin-node-polyfills"

export default defineConfig({
    build: {
        target: "modules",
    },
    plugins: [
        solid(),
        nodePolyfills({
            // To exclude specific polyfills, add them to this list.
            exclude: [
                "fs", // Excludes the polyfill for `fs` and `node:fs`.
            ],
            // Whether to polyfill specific globals.
            globals: {
                Buffer: true, // can also be 'build', 'dev', or false
                global: true,
                process: true,
            },
            // Whether to polyfill `node:` protocol imports.
            protocolImports: true,
        }),
    ],
})
