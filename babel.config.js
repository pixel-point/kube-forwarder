module.exports = function(api) {
  api.cache(true)

  return {
    'comments': false,
    'env': {
      'test': {
        'presets': [
          ['@babel/preset-env', { 'targets': { 'node': 10 } }]
        ]
      },
      'main': {
        'presets': [
          ['@babel/preset-env', { 'targets': { 'node': 10 } }]
        ]
      },
      'renderer': {
        'presets': [
          ['@babel/preset-env', { 'modules': false, 'targets': { 'chrome': 73 } }]
        ]
      },
      'web': {
        // TODO, require configuration after updating to Babel 7.
        'presets': [
          ['@babel/preset-env', { 'modules': false, 'targets': { 'chrome': 73 } }]
        ]
      }
    },
    'plugins': [
      ['@babel/plugin-transform-runtime', { 'corejs': 2 }]
    ]
  }
}
