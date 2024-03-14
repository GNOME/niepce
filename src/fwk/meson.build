
cxx_ncr_bridge = files(meson.project_source_root() / 'crates' / 'npc-craw' / 'src' / 'lib.rs')
cxx_npc_bridge = files(meson.project_source_root() / 'niepce-main' / 'src' / 'lib.rs')

ncr_bindings_cpp = custom_target('ncr_bindings_cpp',
		 output: 'cxx_ncr_bindings.cpp',
		 input: cxx_ncr_bridge,
		 capture: true,
		 command: [cxxbridge, '@INPUT@'])

ncr_bindings_hpp = custom_target('ncr_bindings_hpp',
		 output: 'cxx_ncr_bindings.hpp',
		 input: cxx_ncr_bridge,
		 capture: true,
		 command: [cxxbridge, '--header', '@INPUT@'])

npc_bindings_cpp = custom_target('npc_bindings_cpp',
		 output: 'cxx_npc_bindings.cpp',
		 input: cxx_npc_bridge,
		 capture: true,
		 command: [cxxbridge, '@INPUT@'])

npc_bindings_hpp = custom_target('npc_bindings_hpp',
		 output: 'cxx_npc_bindings.hpp',
		 input: cxx_npc_bridge,
		 capture: true,
		 command: [cxxbridge, '--header', '@INPUT@'])

cxx_headers = declare_dependency(
  sources: [
    npc_bindings_hpp,
    ncr_bindings_hpp,
  ]
)

fwk = static_library(
  'fwk',
  'base' / 'debug.hpp',
  'base' / 'debug.cpp',
  'utils' / 'pathutils.hpp',
  'utils' / 'pathutils.cpp',

  install: false,
  dependencies: [cxx_headers, exempi_deps, gphoto2_deps, adw_deps, gtkmm_deps, shumate_deps],
  include_directories: [include_directories('..', '..' / '..'), rust_inc]
)
