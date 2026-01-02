# Building Niepce

Niepce build system is written with
[meson](https://mesonbuild.com). meson will wrap cargo for the Rust
code, and build the rest.

Make sure to get the git submodules. `git submodule update --init`
should do it.

To build with the address sanitizer, pass `-Db_sanitize=address` to
meson (it's standard).

## Dependencies

- A C++17 compiler
- rustc >= 1.85
- cairo 1.1
- shumate 1.0.0
- libadwaita >= 1.4.0
- exempi >= 2.6.0
- gegl >= 0.4.0
- babl
- libgphoto2
- gexiv2 >= 0.14 (as required by rexiv2)
- libheif
  - HEVC codec should be installed at runtime for HEIC.
- gstreamer-1.0
- meson >= 0.59
- blueprint
- gtk4 4.12
- gtksourceview-5
- python 3

For the RawTherapee engine:

- glibmm 2.68
- giomm
- cairomm
- exiv2 ~= 0.27
- expat
- fftw3f
- libiptcdata
- libraw >= 0.21
- lensfun > 0.3
- lcms2

Niepce is being developed on Linux. It should build and work on other
UNIX systems.

## Building as a flatpak

If you want to build using flatpak-builder, use the manifest in
`flatpak/net.figuiere.Niepce.json`. The following will build and install
it in the user installation of flatpak. It requires the GNOME SDK to
be installed.

```shell
$ cd flatpak
$ flatpak-builder --force-clean --ccache  --install --user build-dir net.figuiere.Niepce.json
```

## Build with fenv

You can use [`fenv`](https://gitlab.gnome.org/ZanderBrown/fenv) to
build the flatpak.

```shell
fenv gen flatpak/net.figuiere.Niepce.Devel.json
fenv exec ninja -C _build install
```

Then you can run with `fenv run`.

### Logging

With `fenv` you customise logging with the `--env` finish argument in
the flatpak manifest.

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
