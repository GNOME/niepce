incdir = include_directories('..' / '..', '..')

niepce_lib = static_library('niepce_lib',
    config_h,
    install: false,
    dependencies: [gtk_deps, gdkpixbuf_deps, giomm_deps, glibmm_deps, gegl_deps],
    include_directories: [incdir, rust_inc]
)
