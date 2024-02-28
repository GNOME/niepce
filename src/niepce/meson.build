incdir = include_directories('..' / '..', '..')

# The bundle is needed for Rust executables.
resources_bundle = gnome.compile_resources(
  'npc-resources',
  'gresource.xml',
  gresource_bundle: true,
  source_dir: meson.project_source_root()
)

niepce_lib = static_library('niepce_lib',
    ncr_bindings_cpp,
    npc_bindings_cpp,
    'ui' / 'dialogs' / 'editlabels.cpp',
    'ui' / 'dialogs' / 'editlabels.hpp',
    'ui' / 'dialogs' / 'preferencesdialog.cpp',
    'ui' / 'dialogs' / 'preferencesdialog.hpp',
    'ui' / 'gridviewmodule.cpp',
    'ui' / 'gridviewmodule.hpp',
    'ui' / 'ilibrarymodule.hpp',
    'ui' / 'metadatapanecontroller.cpp',
    'ui' / 'metadatapanecontroller.hpp',
    'ui' / 'niepceapplication.cpp',
    'ui' / 'niepceapplication.hpp',
    config_h,
    install: false,
    dependencies: [cxx_headers, exempi_deps, gexiv2_deps, gphoto2_deps, gtkmm_deps, sqlite_deps],
    include_directories: [incdir, rust_inc]
)
