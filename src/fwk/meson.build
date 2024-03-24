
cxx_npc_bridge = files(meson.project_source_root() / 'niepce-main' / 'src' / 'lib.rs')

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
  ]
)
