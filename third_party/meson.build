
gtest = static_library(
  'gtest',
  'googletest/googletest/src/gtest_main.cc',
  'googletest/googletest/src/gtest-all.cc',
  include_directories: [ gtest_inc, include_directories('googletest' / 'googletest' ) ]
)

cxxbridge = custom_target(
  'cxxbridge-build',
  output: 'cxxbridge',
  console: true,
  install: false,
  command: [
    'env',
    cargo_env,
    cargo,
    'build', '--manifest-path', '@SOURCE_ROOT@' / 'third_party' / 'cxx' / 'gen' / 'cmd' / 'Cargo.toml',
    '&&',
    'cp', cargo_target_dir / 'debug' / 'cxxbridge',
    '@OUTDIR@'
  ]
)

subdir('rtengine')
