npc_python_blueprints = custom_target('npc-python-blueprints',
  input: files(
    'src/editor.blp',
  ),
  output: '.',
  command: [find_program('blueprint-compiler'), 'batch-compile', '@OUTPUT@', '@CURRENT_SOURCE_DIR@', '@INPUT@'],
)


# The bundle is needed for Rust executables.
npc_python_resources_bundle = gnome.compile_resources(
  'npc-python-resources',
  'gresource.xml',
  gresource_bundle: true,
  source_dir: [
    meson.project_source_root(),
    meson.project_build_root(),
  ],
  dependencies: npc_python_blueprints,
)
