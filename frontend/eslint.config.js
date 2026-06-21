// eslint.config.js
module.exports = [
  {
    ignores: ['android/**', 'ios/**', 'node_modules/**', 'dist/**', 'web-build/**'],
  },
  {
    rules: {
      semi: 'error',
      'prefer-const': 'error',
    },
  },
];
