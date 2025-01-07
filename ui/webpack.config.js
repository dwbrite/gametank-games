const path = require('path');
const CopyWebpackPlugin = require('copy-webpack-plugin');

module.exports = (env, argv) => ({
    entry: './src/main.coffee',
    output: {
        filename: 'bundle.js',
        path: path.resolve(__dirname, '../target/ui/js'),
    },
    module: {
        rules: [
            {
                test: /\.coffee$/,
                use: [
                    {
                        loader: 'babel-loader',
                        options: {
                            presets: ['@babel/preset-env', '@babel/preset-react'],
                            plugins: [
                                [
                                    "@babel/plugin-transform-react-jsx",
                                    {
                                        "pragma": "m",
                                        "pragmaFrag": "'['"
                                    }
                                ]
                            ]
                        },
                    },
                    {
                        loader: 'coffee-loader',
                    },
                ],
            },
            {
                test: /\.s[ac]ss$/,
                use: ['style-loader', 'css-loader', 'sass-loader'],
            },
            {
                test: /\.html$/,
                use: ['html-loader'],
            },
        ],
    },
    plugins: [
        new CopyWebpackPlugin({
            patterns: [
                { from: 'public', to: '../ui' }, // Copy static assets
            ],
        }),
    ],
    resolve: {
        extensions: ['.coffee', '.js', '.jsx', '.html'], // Add .html for resolution
        fallback: {
            "fs": false, // No browser polyfill for `fs`
            "path": require.resolve("path-browserify"),
            "vm": require.resolve("vm-browserify"),
            "module": false,
            "child_process": false,
        },
    },
    devtool: argv.mode === 'development' ? 'source-map' : false,
    watch: argv.mode === 'development', // Watch files in development mode
});
