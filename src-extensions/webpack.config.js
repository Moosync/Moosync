const path = require("node:path");

module.exports = {
	entry: "./src/index.ts",
	mode: "development",
	target: "node",
	module: {
		parser: {
			javascript: {
				commonjsMagicComments: true,
			},
		},
		rules: [
			{
				test: /\.tsx?$/,
				use: "ts-loader",
				exclude: /node_modules/,
			},
		],
	},
	resolve: {
		extensions: [".tsx", ".ts", ".js"],
	},
	output: {
		filename: "main.js",
		path: path.resolve(__dirname, "dist"),
		// libraryTarget: "commonjs2",
	},
};
