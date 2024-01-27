

ncr = static_library(
    'ncr',
    ncr_bindings_cpp,
    'image.cpp',
    'image.hpp',
    'init.cpp',
    'init.hpp',

    dependencies: [cxx_headers, gegl_deps, gtkmm_deps],
    include_directories: include_directories('..')
)
