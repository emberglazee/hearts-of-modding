import path from 'node:path'
import globals from 'globals'
import tseslint, { type FlatConfig } from 'typescript-eslint'
import stylistic from '@stylistic/eslint-plugin'
import eslint from '@eslint/js'

function stylisticRules(): FlatConfig.Rules {
    return {
        'no-trailing-spaces': 'error',
        'eol-last': 'error',
        '@stylistic/semi': ['error', 'never'],
        'arrow-parens': ['error', 'as-needed'],
        'comma-dangle': ['error', 'never'],
        '@stylistic/member-delimiter-style': [
            'error',
            {
                multiline: {
                    delimiter: 'none',
                    requireLast: false
                },
                singleline: {
                    delimiter: 'comma',
                    requireLast: false
                }
            }
        ],
        '@stylistic/space-infix-ops': ['error'],
        'space-before-function-paren': [
            'error',
            {
                anonymous: 'always',
                named: 'never',
                asyncArrow: 'always'
            }
        ],
        quotes: ['error', 'single', { avoidEscape: true }]
    }
}

export default tseslint.config(
    {
        ignores: ['out/']
    },
    eslint.configs.recommended,
    tseslint.configs.recommended,
    {
        files: ['eslint.config.ts'],
        languageOptions: {
            globals: {
                ...globals.node
            },
            parserOptions: {
                project: './tsconfig.eslint.json',
                tsconfigRootDir: path.resolve(),
                ecmaVersion: 'latest',
                sourceType: 'module'
            },
            parser: tseslint.parser
        },
        plugins: {
            '@stylistic': stylistic,
            '@typescript-eslint': tseslint.plugin
        },
        rules: stylisticRules()
    },
    {
        files: ['esbuild.js'],
        languageOptions: {
            globals: {
                ...globals.node,
                ...globals.commonjs
            },
            ecmaVersion: 'latest',
            sourceType: 'commonjs'
        },
        rules: {
            'no-undef': 'off',
            '@typescript-eslint/no-require-imports': 'off',
            'no-unused-expressions': 'off',
            'no-trailing-spaces': 'error',
            'eol-last': 'error',
            quotes: ['error', 'single', { avoidEscape: true }]
        }
    },
    {
        files: ['src/**/*.{js,mjs,cjs,ts}'],
        languageOptions: {
            globals: {
                ...globals.node
            },
            parserOptions: {
                project: './tsconfig.json',
                tsconfigRootDir: path.resolve(),
                ecmaVersion: 'latest',
                sourceType: 'module'
            },
            parser: tseslint.parser
        },
        plugins: {
            '@stylistic': stylistic,
            '@typescript-eslint': tseslint.plugin
        },
        rules: {
            ...stylisticRules(),
            'no-async-promise-executor': 'off',
            'no-case-declarations': 'off',
            '@typescript-eslint/no-empty-object-type': 'off',
            '@typescript-eslint/no-unused-vars': [
                'error',
                {
                    argsIgnorePattern: '^_',
                    varsIgnorePattern: '^_'
                }
            ],
            '@typescript-eslint/no-unused-expressions': 'off'
        }
    }
)
