module.exports = function (api) {
  api.cache(true);
  const plugins = [
    '@babel/plugin-proposal-export-namespace-from',
    'react-native-reanimated/plugin',
  ];

  return {
    presets: [['babel-preset-expo', { jsxImportSource: 'nativewind' }], 'nativewind/babel'],

    plugins,
  };
};
