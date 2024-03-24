incdir = include_directories('..' / '..', '..')

# The bundle is needed for Rust executables.
resources_bundle = gnome.compile_resources(
  'npc-resources',
  'gresource.xml',
  gresource_bundle: true,
  source_dir: meson.project_source_root()
)

niepce_lib = static_library('niepce_lib',
    config_h,
    install: false,
    dependencies: [gtk_deps, gdkpixbuf_deps, giomm_deps, glibmm_deps, gegl_deps],
    include_directories: [incdir, rust_inc]
)
