import js from '@eslint/js';
import globals from 'globals';
import tseslint from 'typescript-eslint';
import pluginVue from 'eslint-plugin-vue';
import {defineConfig} from 'eslint/config';
import stylistic from '@stylistic/eslint-plugin';
import eslintPluginPrettier from 'eslint-plugin-prettier/recommended';

export default defineConfig([
  tseslint.configs.recommended,
  eslintPluginPrettier.configs.recommended,
  stylistic.configs.customize({
    indent: 2,
    quotes: 'single',
    semi: true,
    jsx: true,
    braceStyle: '1tbs',
    arrowParens: 'always',
  }),
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
  }
]);
