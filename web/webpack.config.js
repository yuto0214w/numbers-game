const path = require("node:path");

const HtmlWebpackPlugin = require("html-webpack-plugin");
const HtmlInlineScriptPlugin = require("html-inline-script-webpack-plugin");

/** @type {import("webpack").Configuration} */
const config = {
  mode: "production",
  entry: {
    index: path.join(__dirname, "src", "index.js"),
    main: path.join(__dirname, "src", "game.js"),
  },
  module: {
    rules: [
      {
        test: /\.js$/,
        exclude: /node_modules/,
        use: {
          loader: "babel-loader",
          options: {
            presets: [["@babel/preset-env", { targets: "defaults" }]],
          },
        },
      },
    ],
  },
  plugins: [
    new HtmlWebpackPlugin({
      template: path.join(__dirname, "src", "index.html"),
      filename: "index.html",
      chunks: ["index"],
      inject: "body",
      scriptLoading: "blocking",
      minify: false,
    }),
    new HtmlWebpackPlugin({
      template: path.join(__dirname, "src", "game.html"),
      filename: "game.html",
      chunks: ["main"],
      inject: "head",
      scriptLoading: "blocking",
      minify: false,
    }),
    new HtmlInlineScriptPlugin({
      scriptMatchPattern: [/^index\.js$/],
    }),
  ],
};

module.exports = config;
