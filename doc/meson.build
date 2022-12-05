
doxyfile_conf = configuration_data()
doxyfile_conf.set('top_srcdir', meson.project_source_root())

doxygen = find_program('doxygen', required: false)

doxyfile = configure_file(
  input: 'Doxyfile.in',
  output: 'Doxyfile',
  configuration: doxyfile_conf
)

if doxygen.found()
  dox = run_target(
    'dox',
    command: [
      doxygen,
      doxyfile
    ]
  )
endif
