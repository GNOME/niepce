
cxx_colour_bridge = files(meson.project_source_root() / 'crates' / 'npc-fwk' / 'src' / 'base' / 'rgbcolour.rs')
cxx_fwk_bridge = files(meson.project_source_root() / 'crates' / 'npc-fwk' / 'src' / 'lib.rs')
cxx_widgets_bridge = files(meson.project_source_root() / 'crates' / 'npc-fwk' / 'src' / 'toolkit' / 'widgets' / 'metadata_widget.rs')
cxx_eng_bridge = files(meson.project_source_root() / 'crates' / 'npc-engine' / 'src' / 'lib.rs')
cxx_ncr_bridge = files(meson.project_source_root() / 'crates' / 'npc-craw' / 'src' / 'lib.rs')
cxx_npc_bridge = files(meson.project_source_root() / 'niepce-main' / 'src' / 'lib.rs')

colour_bindings_cpp = custom_target('colour_bindings_cpp',
		 output: 'cxx_colour_bindings.cpp',
		 input: cxx_colour_bridge,
		 capture: true,
		 command: [cxxbridge, '@INPUT@'])

colour_bindings_hpp = custom_target('colour_bindings_hpp',
		 output: 'cxx_colour_bindings.hpp',
		 input: cxx_colour_bridge,
		 capture: true,
		 command: [cxxbridge, '--header', '@INPUT@'])

widgets_bindings_cpp = custom_target('widgets_bindings_cpp',
		 output: 'cxx_widgets_bindings.cpp',
		 input: cxx_widgets_bridge,
		 capture: true,
		 command: [cxxbridge, '@INPUT@'])

widgets_bindings_hpp = custom_target('widgets_bindings_hpp',
		 output: 'cxx_widgets_bindings.hpp',
		 input: cxx_widgets_bridge,
		 capture: true,
		 command: [cxxbridge, '--header', '@INPUT@'])

fwk_bindings_cpp = custom_target('fwk_bindings_cpp',
		 output: 'cxx_fwk_bindings.cpp',
		 input: cxx_fwk_bridge,
		 capture: true,
		 command: [cxxbridge, '@INPUT@'])

fwk_bindings_hpp = custom_target('fwk_bindings_hpp',
		 output: 'cxx_fwk_bindings.hpp',
		 input: cxx_fwk_bridge,
		 capture: true,
		 command: [cxxbridge, '--header', '@INPUT@'])

eng_bindings_cpp = custom_target('eng_bindings_cpp',
		 output: 'cxx_eng_bindings.cpp',
		 input: cxx_eng_bridge,
		 capture: true,
		 command: [cxxbridge, '@INPUT@'])

eng_bindings_hpp = custom_target('eng_bindings_hpp',
		 output: 'cxx_eng_bindings.hpp',
		 input: cxx_eng_bridge,
		 capture: true,
		 command: [cxxbridge, '--header', '@INPUT@'])

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
    fwk_bindings_hpp,
    eng_bindings_hpp,
    npc_bindings_hpp,
    ncr_bindings_hpp,
    colour_bindings_hpp,
    widgets_bindings_hpp,
  ]
)

fwk = static_library(
  'fwk',
  fwk_bindings_cpp,
  eng_bindings_cpp,
  colour_bindings_cpp,
  widgets_bindings_cpp,
  'base' / 'autoflag.hpp',
  'base' / 'debug.hpp',
  'base' / 'debug.cpp',
  'base' / 'geometry.hpp',
  'base' / 'geometry.cpp',
  'base' / 'map.hpp',
  'base' / 'propertybag.hpp',
  'base' / 'propertybag.cpp',
  'base' / 'singleton.hpp',
  'base' / 'util.hpp',
  'toolkit' / 'appframe.hpp',
  'toolkit' / 'appframe.cpp',
  'toolkit' / 'application.hpp',
  'toolkit' / 'application.cpp',
  'toolkit' / 'command.hpp',
  'toolkit' / 'configdatabinder.hpp',
  'toolkit' / 'configdatabinder.cpp',
  'toolkit' / 'controller.hpp',
  'toolkit' / 'controller.cpp',
  'toolkit' / 'dialog.hpp',
  'toolkit' / 'dialog.cpp',
  'toolkit' / 'dockable.hpp',
  'toolkit' / 'dockable.cpp',
  'toolkit' / 'frame.hpp',
  'toolkit' / 'frame.cpp',
  'toolkit' / 'gdkutils.hpp',
  'toolkit' / 'gdkutils.cpp',
  'toolkit' / 'gtkutils.hpp',
  'toolkit' / 'gtkutils.cpp',
  'toolkit' / 'mapcontroller.hpp',
  'toolkit' / 'mapcontroller.cpp',
  'toolkit' / 'uicontroller.hpp',
  'toolkit' / 'uicontroller.cpp',
  'toolkit' / 'widgets' / 'dock.hpp',
  'toolkit' / 'widgets' / 'dock.cpp',
  'toolkit' / 'widgets' / 'editablehscale.hpp',
  'toolkit' / 'widgets' / 'editablehscale.cpp',
  'toolkit' / 'widgets' / 'toolboxitemwidget.hpp',
  'toolkit' / 'widgets' / 'toolboxitemwidget.cpp',
  'utils' / 'databinder.hpp',
  'utils' / 'databinder.cpp',
  'utils' / 'exempi.hpp',
  'utils' / 'exempi.cpp',
  'utils' / 'pathutils.hpp',
  'utils' / 'pathutils.cpp',
  'utils' / 'ufrawmeta.hpp',
  'utils' / 'ufrawmeta.cpp',

  install: false,
  dependencies: [cxx_headers, exempi_deps, gphoto2_deps, adw_deps, gtkmm_deps, libxml_deps, shumate_deps],
  include_directories: [include_directories('..', '..' / '..'), rust_inc]
)
