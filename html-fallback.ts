import type { Plugin } from "vite";
import path from "path";
import fs from "fs";

export interface Options {
	rootDir: string;
}

export default (options?: Options): Plugin => ({
	name: "html-ext-fallback",
	configureServer(server) {
		server.middlewares.use((req, res, next) => {
			// biome-ignore lint/style/noNonNullAssertion: <explanation>
			// biome-ignore lint/suspicious/noExtraNonNullAssertion: <explanation>
			if (req.originalUrl!!.length > 1 && !path.extname(req.originalUrl!!)) {
				if (
					fs.existsSync(
						path.join(options?.rootDir ?? __dirname, `${req.originalUrl}.html`),
					)
				) {
					req.url += ".html";
				}
			}

			next();
		});
	},
});
