import js from '@eslint/js';
import globals from 'globals';
import tseslint from 'typescript-eslint';
import pluginVue from 'eslint-plugin-vue';
import {defineConfig} from 'eslint/config';
import stylistic from '@stylistic/eslint-plugin';


export default defineConfig([
  tseslint.configs.recommended,
  pluginVue.configs['flat/essential'],
  { 
    files: ['**/*.{js,mjs,cjs,ts,vue}'], 
    plugins: {js}, extends: ['js/recommended']},
  {
    files: ['**/*.{js,mjs,cjs,ts,vue}'], 
    languageOptions: {globals: globals.browser}},
  {
    files: ['**/*.vue'], 
    languageOptions: {parserOptions: {parser: tseslint.parser}}},
  // ts无全局类型报错
  {
    files:['**/*.{ts,vue}'],
    rules:{
      'no-undef':'off'
    }
  },
  // 代码格式化配置
  {
    files: ['**/*.{js,mjs,cjs,ts,vue}'], 
    plugins: {'@stylistic': stylistic},
    rules: {
      '@stylistic/indent': ['error', 2],
      '@stylistic/max-len':['error', {'code':80}],
      '@stylistic/semi': ['error', 'always'],
      '@stylistic/arrow-parens':['error', 'always'],
      '@stylistic/comma-dangle': ['error', 'never'],
      '@stylistic/quotes': ['error', 'single'],
      '@stylistic/space-in-parens': ['error', 'never'],
      '@stylistic/no-floating-decimal': 'error',
      '@stylistic/block-spacing': ['error', 'never'],
      '@stylistic/array-bracket-spacing': ['error', 'never'],
      '@stylistic/object-curly-spacing': ['error', 'never'],
      '@stylistic/no-multi-spaces': ['error'],
      '@stylistic/space-infix-ops': ['error']
    }
  }
]);