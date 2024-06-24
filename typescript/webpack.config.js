const path = require("path");
const build = path.join(process.cwd(), "build");
const FilterWebpackOutput = require("filter-webpack-output");
module.exports = env => ({
    entry: "./src/index.tsx",
    devtool: env == "prod" ? undefined : "inline-source-map",
    mode: env == "prod" ? "production" : "development",
    module: {
        rules: [
            {
                test: /(?<!\.d)\.tsx?$/,
                use: "ts-loader",
                exclude: /node_modules/,
            },
            {
                test: /\.css$/i,
                use: ["style-loader", "css-loader"],
            },
            {
                test: /\.txt$/i,
                use: "raw-loader",
            },
            {}
        ],
    },
    experiments: {
        asyncWebAssembly: true
    },
    devServer: {
        static: {
            directory: build
        },
        compress: true,
        port: 3000,
        historyApiFallback: true,
    },
    watchOptions: {
      aggregateTimeout: 1200,
    },
    resolve: {
        extensions: [".tsx", ".ts", ".d.ts", ".js", ".txt"],
        alias: {
            "vis-network": "vis-network/standalone",
        },
    },
    output: {
        filename: "bundle.js",
        path: build,
    },
    plugins: [
        new FilterWebpackOutput(/(vendors-)?node_modules_monaco-editor/), // You can also pass as array of RegExp
    ],
});
