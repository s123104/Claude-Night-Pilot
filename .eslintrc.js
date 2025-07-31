module.exports = {
  env: {
    browser: true,
    es2021: true,
  },
  extends: [
    'eslint:recommended',
  ],
  parserOptions: {
    ecmaVersion: 12,
    sourceType: 'module',
  },
  globals: {
    // Tauri globals
    '__TAURI_API__': 'readonly',
    'invoke': 'readonly',
    
    // Application globals
    'appState': 'writable',
    'themeManager': 'writable',
    'snackbarManager': 'writable',
    'navigationManager': 'writable',
    'modalManager': 'writable',
    'apiClient': 'writable',
    'promptManager': 'writable',
    'jobManager': 'writable',
    'resultManager': 'writable',
    'systemManager': 'writable',
    'cooldownManager': 'writable',
    'unifiedApiClient': 'writable',
    'promptExecutor': 'writable',
  },
  rules: {
    'indent': ['error', 2],
    'quotes': ['error', 'double'],
    'semi': ['error', 'always'],
    'comma-dangle': ['error', 'always-multiline'],
    'no-unused-vars': ['error', { 'argsIgnorePattern': '^_' }],
    'no-console': 'warn',
    'curly': ['error', 'all'],
  },
};