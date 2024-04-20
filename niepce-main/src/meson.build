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

cargo_options = [ '--manifest-path', meson.project_source_root() / 'Cargo.toml' ]
cargo_options += [ '--target-dir', cargo_target_dir ]

if get_option('profile') == 'release'
  cargo_options += [ '--release' ]
  rust_target = 'release'
  message('Building in release mode')
else
  rust_target = 'debug'
  message('Building in debug mode')
endif

# cargo env is defined in the top-level

niepce_rust = custom_target(
  'cargo-build',
  build_always_stale: true,
  output: [
    'niepce'
  ],
  console: true,
  install: true,
  install_dir: 'bin',
  command: [
    'env',
    cargo_env,
    cargo, 'build',
    cargo_options,
    '&&',
    'cp', cargo_target_dir / rust_target / 'niepce', '@OUTDIR@'
  ],
  depends: [niepce_lib, resources_bundle, rtengine]
)

clippy = run_target(
  'clippy',
  env: cargo_env,
  command: [
    cargo, 'clippy',
    cargo_options,
    '--all-targets'
  ],
  depends: [resources_bundle]
)

rustdoc = run_target(
  'rustdoc',
  env: cargo_env,
  command: [
    cargo, 'doc',
    cargo_options,
  ],
)

examples = run_target(
  'examples',
  env: cargo_env,
  command: [
    cargo, 'build',
    cargo_options,
    '--examples'
  ],
  depends: [resources_bundle]
)

niepce_rust_deps = declare_dependency(link_with: niepce_rust[0])

# Target to build the test prior to running cargo-test
niepce_test = custom_target(
  'cargo-test-build',
  console: true,
  install: false,
  output: [
    'test-build-stamp'
  ],
  command: [
    'env',
    cargo_env,
    cargo, 'build',
    cargo_options,
    '--tests',
    '&&',
    'touch', '@OUTDIR@' / 'test-build-stamp'
  ],
  depends: [niepce_lib, resources_bundle, rtengine]
)

test(
  'cargo-test',
  cargo,
  env: cargo_env,
  args: [
    'test',
    cargo_options
  ],
  depends: [niepce_lib, resources_bundle, rtengine],
  timeout: 0
)
