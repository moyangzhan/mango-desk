module.exports = {
  root: true,
  rules: {
    'no-console': process.env.NODE_ENV === 'production' ? 'error' : 'off',
    'no-debugger': process.env.NODE_ENV === 'production' ? 'error' : 'off',
    '@typescript-eslint/brace-style': ['error', '1tbs', { allowSingleLine: true }],
    'vue/v-on-event-hyphenation': ['error', 'always'],
    'vue/attribute-hyphenation': ['error', 'always'],
    'vue/require-v-for-key': 'error',
    'vue/component-name-in-template-casing': ["error", "PascalCase", {
      "ignores": [],
      "registeredComponentsOnly": false
    }],
    'vue/no-setup-props-destructure': 'off',
  },
  extends: ['@antfu'],
}
