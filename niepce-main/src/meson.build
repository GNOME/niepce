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

blueprints = custom_target('blueprints',
  input: files(
    'niepce/ui/niepce_window.blp',
    'niepce/ui/dialogs/import/camera_importer_ui.blp',
    'niepce/ui/dialogs/import/directory_importer_ui.blp',
    'niepce/ui/dialogs/import.blp',
    'niepce/ui/dialogs/preferences_dialog.blp',
  ),
  output: '.',
  command: [find_program('blueprint-compiler'), 'batch-compile', '@OUTPUT@', '@CURRENT_SOURCE_DIR@', '@INPUT@'],
)

# The bundle is needed for Rust executables.
resources_bundle = gnome.compile_resources(
  'npc-resources',
  'niepce/gresource.xml',
  gresource_bundle: true,
  source_dir: [
    meson.project_source_root(),
    meson.project_build_root(),
  ],
  dependencies: blueprints,
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
  depends: [resources_bundle, rtengine]
)

test(
  'cargo-test',
  cargo,
  env: cargo_env,
  args: [
    'test',
    cargo_options,
    '--',
    '--nocapture'
  ],
  depends: [resources_bundle, rtengine],
  timeout: 0
)
