

incdir = include_directories('..' / '..', '..')

# Resources
resources = gnome.compile_resources(
  'resources',
  'gresource.xml',
  c_name: 'npc',
  source_dir: meson.project_source_root()
)

# The bundle is needed for the widget-demo
# run by the test.
resources_bundle = gnome.compile_resources(
  'npc-resources',
  'gresource.xml',
  gresource_bundle: true,
  source_dir: meson.project_source_root()
)

executable('niepce',
    npc_bindings_cpp,
    'main.cpp',
    'modules' / 'darkroom' / 'darkroommodule.cpp',
    'modules' / 'darkroom' / 'darkroommodule.hpp',
    'modules' / 'darkroom' / 'dritemwidget.cpp',
    'modules' / 'darkroom' / 'dritemwidget.hpp',
    'modules' / 'darkroom' / 'imagecanvas.cpp',
    'modules' / 'darkroom' / 'imagecanvas.hpp',
    'modules' / 'darkroom' / 'toolboxcontroller.cpp',
    'modules' / 'darkroom' / 'toolboxcontroller.hpp',
    'modules' / 'interfaces' / 'ipostimportprocessing.cpp',
    'modules' / 'interfaces' / 'ipostimportprocessing.hpp',
    'modules' / 'map' / 'mapmodule.cpp',
    'modules' / 'map' / 'mapmodule.hpp',
    'ui' / 'dialogs' / 'editlabels.cpp',
    'ui' / 'dialogs' / 'editlabels.hpp',
    'ui' / 'dialogs' / 'importdialog.cpp',
    'ui' / 'dialogs' / 'importdialog.hpp',
    'ui' / 'dialogs' / 'importers' / 'cameraimporterui.cpp',
    'ui' / 'dialogs' / 'importers' / 'cameraimporterui.hpp',
    'ui' / 'dialogs' / 'importers' / 'directoryimporterui.cpp',
    'ui' / 'dialogs' / 'importers' / 'directoryimporterui.hpp',
    'ui' / 'dialogs' / 'importers' / 'iimporterui.hpp',
    'ui' / 'dialogs' / 'importers' / 'importerui.cpp',
    'ui' / 'dialogs' / 'importers' / 'importerui.hpp',
    'ui' / 'dialogs' / 'preferencesdialog.cpp',
    'ui' / 'dialogs' / 'preferencesdialog.hpp',
    'ui' / 'gridviewmodule.cpp',
    'ui' / 'gridviewmodule.hpp',
    'ui' / 'ilibrarymodule.hpp',
    'ui' / 'metadatapanecontroller.cpp',
    'ui' / 'metadatapanecontroller.hpp',
    'ui' / 'niepceapplication.cpp',
    'ui' / 'niepceapplication.hpp',
    resources,
    config_h,
    install: true,
    install_dir: bindir,
    link_with: [fwk, engine, ncr],
    dependencies: [cxx_headers, exempi_deps, gexiv2_deps, gphoto2_deps, gtkmm_deps, sqlite_deps, niepce_rust_deps],
    include_directories: [incdir, rust_inc]
)
