
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
  depends: [resources_bundle, rtengine]
)
