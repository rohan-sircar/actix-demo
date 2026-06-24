module.exports = function (api) {
  api.cache(true);
  const plugins = [
    '@babel/plugin-proposal-export-namespace-from',
    'react-native-reanimated/plugin',
  ];

  return {
    presets: [['babel-preset-expo', { jsxImportSource: 'nativewind', unstable_transformImportMeta: true }], 'nativewind/babel'],

    plugins,
  };
};
