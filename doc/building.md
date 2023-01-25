Building Niepce
===============

Niepce build system is written with
[meson](https://mesonbuild.com). meson will wrap cargo for the Rust
code, and build the rest.

The autotools system is deprecated and *may* be broken.

## Build profile

Pass `-Dprofile=` to meson to set the build profile. As a packager you
want to pas `-Dprofile=release`.

The default build profile is `development`. The other is
`release`. The build profile `development` changes the following:

- The app-id is suffixed with `.Devel`.
- Rust code is build in debug.
- The app configuration directory is `niepce-devel`.
- It has a different icon.
- The window has "stripes".
- Define the const `PROFILE` or `config::PROFILE` to `Devel`.

This allow to not default on the same catalog as when you use the
release package when Niepce reopen the last one.
