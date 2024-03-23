
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
