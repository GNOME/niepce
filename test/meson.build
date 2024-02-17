# test

# base
testgeometry = executable(
  'testgeometry',
  '..' / 'src' / 'fwk' / 'base' / 't' / 'testgeometry.cpp',
  build_by_default: false,
  install: false,
  include_directories: [ gtest_inc, include_directories( '..' / 'src', '..' / 'src' / 'fwk' / 'base') ],
  link_with: [ gtest, fwk ]
)
test('Test base/geometry', testgeometry)

# toolkit
testconfigdatabinder = executable(
  'testconfigdatabinder',
  '..' / 'src' / 'fwk' / 'toolkit' / 't' / 'testconfigdatabinder.cpp',
  fwk_bindings_cpp,
  eng_bindings_cpp,
  colour_bindings_cpp,
  widgets_bindings_cpp,
  npc_bindings_cpp,
  build_by_default: false,
  install: false,
  dependencies: [
    cxx_headers, gtkmm_deps, exempi_deps, gexiv2_deps, niepce_rust_deps,
    # For rtengine
    png16_deps, tiff_deps, jpeg_deps, expat_deps, fftw3f, zlib_deps,
    exiv2_deps
  ],
  include_directories: [ gtest_inc, include_directories('..' / 'src', '..' / 'src' / 'fwk' / 'toolkit') ],
  link_with: [ gtest, fwk, ncr, niepce_lib, rtengine ]
)
test('Test toolkit/configdatabinder', testconfigdatabinder)
