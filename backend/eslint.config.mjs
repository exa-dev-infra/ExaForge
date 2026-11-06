import js from '@eslint/js'
import tseslint from 'typescript-eslint'
import globals from 'globals'

// Strict, standard-style ESLint configuration:
//  - single quotes, no semicolons, 2-space indentation
//  - typescript-eslint recommended (bug-catching) rules
//  - ignoring generated directories
export default tseslint.config(
  {
    ignores: [
      'dist/**',
      'node_modules/**',
      'coverage/**',
      'migrations/**',
      'public/**',
      'logs/**'
    ]
  },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ['**/*.ts'],
    languageOptions: {
      parserOptions: {
        sourceType: 'module'
      },
      globals: {
        ...globals.node
      }
    },
    rules: {
      // ---- Style: always identical regardless of who writes the code ----
      semi: ['error', 'never'],
      quotes: ['error', 'single', {avoidEscape: true}],
      indent: ['error', 2],
      'comma-dangle': ['error', 'never'],
      'space-before-function-paren': ['error', 'always'],
      'object-curly-spacing': ['error', 'always'],
      'array-bracket-spacing': ['error', 'never'],
      'space-infix-ops': 'error',
      'keyword-spacing': ['error', {before: true, after: true}],
      'no-multi-spaces': 'error',
      'max-len': ['warn', {code: 120, ignoreUrls: true, ignoreComments: true}],
      'no-console': 'off',

      // ---- Strictness ----
      eqeqeq: ['error', 'always'],
      curly: ['error', 'all'],
      'no-var': 'error',
      'prefer-const': 'error',
      'prefer-template': 'warn',
      '@typescript-eslint/consistent-type-imports': ['error', {prefer: 'type-imports'}],
      '@typescript-eslint/no-namespace': ['error', {allowDeclarations: true}],
      '@typescript-eslint/no-unused-vars': [
        'error',
        {argsIgnorePattern: '^_', varsIgnorePattern: '^_', caughtErrors: 'none'}
      ],
      '@typescript-eslint/no-explicit-any': 'warn'
    }
  }
)