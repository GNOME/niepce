
engine = static_library(
    'engine',
    'importer' / 'cameraimporter.cpp',
    'importer' / 'cameraimporter.hpp',
    'importer' / 'directoryimporter.cpp',
    'importer' / 'directoryimporter.hpp',
    'importer' / 'iimporter.hpp',

    dependencies: [cxx_headers, exempi_deps, gphoto2_deps, gtkmm_deps, niepce_rust_deps],
    include_directories: [include_directories('..'), rust_inc]
)
