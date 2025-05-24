import antfu from '@antfu/eslint-config'

export default antfu({
  vue: true,
  typescript: false,
  formatters: true,
  stylistic: {
    indent: 2,
    quotes: 'single',
    semi: false,
  },
  rules: {
    // 自定义规则
    'no-console': 'warn',
    'vue/max-attributes-per-line': ['error', {
      singleline: 3,
      multiline: 1,
    }],
    'vue/html-self-closing': ['error', {
      html: {
        void: 'always',
        normal: 'always',
        component: 'always',
      },
      svg: 'always',
      math: 'always',
    }],
  },
})
