

ncr = static_library(
    'ncr',
    ncr_bindings_cpp,
    'init.cpp',
    'init.hpp',

    dependencies: [cxx_headers, gtk_deps, gdkpixbuf_deps, gegl_deps],
    include_directories: include_directories('..')
)
