const path = require("path");
const webpack = require('webpack');
const build = path.join(process.cwd(), "build");
const CopyPlugin = require("copy-webpack-plugin");
module.exports = env => ({
    entry: "./src/index.tsx",
    devtool: env == "prod" ? undefined : "source-map",
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
            }
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
        aggregateTimeout: 2000,
    },
    resolve: {
        extensions: [".tsx", ".ts", ".js", ".txt"],
    },
    output: {
        filename: "bundle.js",
        path: build,
    },
    plugins: [
        // new CopyPlugin({
        //   patterns: [
        //     { from: "node_modules/oxidd-viz-rust/oxidd_viz_rust_bg.wasm.map", to: "" },
        //   ],
        // }),
    ],
});
