// prettier.config.js
/**
 * @type {import('prettier').Config}
 * @see https://www.prettier.cn/docs/options.html
 */
export default {
  trailingComma: 'all',
  singleQuote: true,
  semi: true,
  printWidth: 80,
  arrowParens: 'always',
  proseWrap: 'always',
  experimentalTernaries: false,
  tabWidth: 2,

  useTabs: false,
  quoteProps: 'consistent',
  jsxSingleQuote: false,
  bracketSpacing: true,
  bracketSameLine: false,
  vueIndentScriptAndStyle: false,
  singleAttributePerLine: false
};
