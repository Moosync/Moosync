module.exports = {
  root: true,

  env: {
    node: true,
    browser: true,
    es6: true
  },

  plugins: ['unused-imports'],

  parserOptions: {
    parser: '@typescript-eslint/parser',
    ecmaVersion: 2022
  },

  rules: {
    'no-unused-vars': 0,
    semi: 'off',
    'no-extra-semi': 0,
    'no-undef': 0,
    'unused-imports/no-unused-imports': 'error',
    'no-debugger': process.env.NODE_ENV === 'production' ? 'warn' : 'off',
    '@typescript-eslint/no-namespace': 'off',
    'vue/no-v-for-template-key': 'off'
  },

  extends: [
    'plugin:vue/essential',
    'eslint:recommended',
    '@vue/typescript',
    '@vue/typescript/recommended',
  ]
}
