project(
  'niepce',
  'cpp',
  'rust',
  version: '0.1.0',
  meson_version: '>= 0.59',
  license: 'GPL-3.0-or-later',
  default_options: [
    'cpp_std=c++17'
  ]
)

i18n = import('i18n')
gnome = import('gnome')

base_id = 'org.gnome.Niepce'

dependency('glib-2.0', version: '>= 2.66')
dependency('gio-2.0', version: '>= 2.66')
dependency('gtk4', version: '>= 4.4.0')
adw_deps = dependency('libadwaita-1', version: '>= 1.2.0')
dependency('glibmm-2.68')
gtkmm_deps = dependency('gtkmm-4.0', version: '>= 4.4.0')
# need at least 2.5.0 because of xmlTextReader
libxml_deps = dependency('libxml-2.0', version: '>= 2.5.0')
sqlite_deps = dependency('sqlite3')
exempi_deps = dependency('exempi-2.0', version: '>= 2.4.0')
libopenraw_deps = dependency('libopenraw-0.3', version: '>= 0.3.0')
dependency('boost', version: '>= 1.60')
dependency('babl')
gegl_deps = dependency('gegl-0.4')
gexiv2_deps = dependency('gexiv2')
gphoto2_deps = dependency('libgphoto2', version: '>= 2.5')
shumate_deps = dependency('shumate-1.0', version: '>= 1.0.0')

glib_compile_resources = find_program('glib-compile-resources', required: true)
# glib_compile_schemas = find_program('glib-compile-schemas', required: true)
desktop_file_validate = find_program('desktop-file-validate', required: false)
appstream_util = find_program('appstream-util', required: false)
cargo = find_program('cargo', required: true)

version = meson.project_version()

prefix = get_option('prefix')
bindir = prefix / get_option('bindir')
localedir = prefix / get_option('localedir')

datadir = prefix / get_option('datadir')
pkgdatadir = datadir / meson.project_name()
iconsdir = datadir / 'icons'
podir = meson.project_source_root() / 'po'
gettext_package = meson.project_name()

if get_option('profile') == 'development'
  profile = 'Devel'
  vcs_tag = run_command('git', 'rev-parse', '--short', 'HEAD', check: false).stdout().strip()
  if vcs_tag == ''
    version_suffix = '-devel'
  else
    version_suffix = '-@0@'.format(vcs_tag)
  endif
  application_id = '@0@.@1@'.format(base_id, profile)
  package = '@0@-devel'.format(meson.project_name())
else
  profile = ''
  version_suffix = ''
  application_id = base_id
  package = meson.project_name()
endif

if get_option('buildtype') == 'debug'
  add_project_arguments('-DDEBUG', language: 'cpp')
else
  add_project_arguments('-DNDEBUG', language: 'cpp')
endif

meson.add_dist_script(
  'build-aux/dist-vendor.sh',
  meson.project_build_root() / 'meson-dist' / meson.project_name() + '-' + version,
  meson.project_source_root()
)

if get_option('profile') == 'development'
  # Setup pre-commit hook for ensuring coding style is always consistent
  message('Setting up git pre-commit hook..')
  run_command('cp', '-f', 'hooks/pre-commit.hook', '.git/hooks/pre-commit', check: false)
endif

global_conf = configuration_data()
global_conf.set_quoted('APP_ID', application_id)
global_conf.set_quoted('PKGDATADIR', pkgdatadir)
global_conf.set_quoted('PROFILE', profile)
global_conf.set_quoted('NIEPCE_VERSION', version)
global_conf.set_quoted('VERSION', version + version_suffix)
global_conf.set_quoted('GETTEXT_PACKAGE', gettext_package)
global_conf.set_quoted('LOCALEDIR', localedir)
global_conf.set_quoted('NIEPCE_LOCALEDIR', localedir)
global_conf.set_quoted('NIEPCE_BUILD_CONFIG', '')
global_conf.set_quoted('DATADIR', datadir)
global_conf.set_quoted('PACKAGE', package)

config_h = configure_file(
  input: 'config.h.meson.in',
  output: 'config.h',
  configuration: global_conf
)

rust_inc = include_directories('.')
gtest_inc = include_directories('.' / 'third_party' / 'googletest' / 'googletest' / 'include')

subdir('data')
subdir('po')
subdir('niepce-main/src')
subdir('third_party')
# TODO this should be built as a dependency
subdir('src')
subdir('doc')

gnome.post_install(
  gtk_update_icon_cache: true,
  # We don't have any schemas
  glib_compile_schemas: false,
  update_desktop_database: true,
)
