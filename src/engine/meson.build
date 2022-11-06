
engine = static_library(
    'engine',
    'db' / 'libfile.hpp',
    'db' / 'metadata.hpp',
    'db' / 'properties.cpp',
    'db' / 'properties-def.hpp',
    'db' / 'properties.hpp',
    'importer' / 'cameraimporter.cpp',
    'importer' / 'cameraimporter.hpp',
    'importer' / 'directoryimporter.cpp',
    'importer' / 'directoryimporter.hpp',
    'importer' / 'iimporter.hpp',
    'importer' / 'importedfile.cpp',
    'importer' / 'importedfile.hpp',

    dependencies: [cxx_headers, exempi_deps, gphoto2_deps, gtkmm_deps, niepce_rust_deps],
    include_directories: [include_directories('..'), rust_inc]
)
