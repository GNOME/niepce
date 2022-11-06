config = configure_file(
  input: 'config.rs.in',
  output: 'config.rs',
  configuration: global_conf
)
# Copy the config.rs output to the source directory.
run_command(
  'cp',
  meson.project_build_root() / 'niepce-main' / 'src' / 'config.rs',
  meson.project_source_root() / 'niepce-main' / 'src' / 'config.rs',
  check: true
)

cargo_target_dir = meson.project_build_root() / 'target'
cargo_options = [ '--manifest-path', meson.project_source_root() / 'Cargo.toml' ]
cargo_options += [ '--target-dir', cargo_target_dir ]

if get_option('profile') == 'default'
  cargo_options += [ '--release' ]
  rust_target = 'release'
  message('Building in release mode')
else
  rust_target = 'debug'
  message('Building in debug mode')
endif

cargo_env = [
  'CARGO_HOME=' + meson.project_build_root() / 'cargo-home',
  'CARGO_TARGET_DIR=' + cargo_target_dir
]

niepce_rust = custom_target(
  'cargo-build',
  build_always_stale: true,
  output: [
    'libniepce_rust.a',
    'bindings.h',
    'eng_bindings.h'
  ],
  console: true,
  install: false,
  command: [
    'env',
    cargo_env,
    cargo, 'build',
    cargo_options,
    '&&',
    'cp', cargo_target_dir / rust_target / 'libniepce_rust.a', '@OUTDIR@'
  ]
)

niepce_rust_deps = declare_dependency(link_with: niepce_rust[0], sources: [ niepce_rust[1], niepce_rust[2] ])

test(
  'cargo-test',
  cargo,
  env: cargo_env,
  args: [
    'test',
    cargo_options
  ]
)
