import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import path, { dirname, resolve } from "path";
import json from "@rollup/plugin-json";
import VueI18nPlugin from "@intlify/unplugin-vue-i18n/vite";
import { fileURLToPath } from "url";
import { internalIpV4 } from "internal-ip";
import HtmlExtFallbackPlugin from "./html-fallback";

// https://vitejs.dev/config/
export default defineConfig(async () => ({
	plugins: [
		HtmlExtFallbackPlugin({ rootDir: __dirname }),
		vue({
			template: {
				compilerOptions: {
					compatConfig: {
						MODE: 2,
					},
				},
			},
		}),
		VueI18nPlugin({
			include: resolve(
				dirname(fileURLToPath(import.meta.url)),
				"./src/utils/ui/i18n/*.json",
			),
			strictMessage: false,
			escapeHtml: true,
		}),
	],
	resolve: {
		alias: {
			"@": path.resolve(__dirname, "src"),
			vue: "@vue/compat",
		},
	},
	build: {
		rollupOptions: {
			input: {
				mainWindow: "./mainWindow.html",
				preferenceWindow: "./preferenceWindow.html",
			},
		},
	},
	define: {
		"process.env": process.env,
	},

	// Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
	//
	// 1. prevent vite from obscuring rust errors
	clearScreen: false,
	// 2. tauri expects a fixed port, fail if that port is not available
	server: {
		port: 1420,
		host: true,
		strictPort: true,
		watch: {
			// 3. tell vite to ignore watching `src-tauri`
			ignored: ["**/src-tauri/**"],
		},
		hmr: {
			protocol: "ws",
			host: "192.168.1.162",
			port: 5184,
		},
	},
}));
