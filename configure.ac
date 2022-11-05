AC_INIT(niepce, 2022.5.1)
AC_PREREQ(2.59)
AC_CONFIG_SRCDIR(README)
AM_INIT_AUTOMAKE([1.11 no-dist-gzip dist-xz subdir-objects])
m4_ifndef([AM_SILENT_RULES], [m4_define([AM_SILENT_RULES],[])])
AM_SILENT_RULES([yes])
AC_CONFIG_HEADER(config.h)
AC_CONFIG_MACRO_DIR(m4)

m4_pattern_allow([^BOOST_])

dnl all the library version.
dnl if one is harcoded elsewhere, it is a bug
LIBGIOMM_VERSION=2.66
LIBGLIBMM_VERSION=2.66
LIBGTKMM_VERSION=4.4
EXEMPI_VERSION=2.4.0
SQLITE_VERSION=3.0
GEGL_VERSION=0.3.0
LIBOPENRAW_VERSION=0.3.0
LIBGPHOTO_VERSION=2.5
dnl need at least 2.5.0 because of xmlTextReader
LIBXML2_VERSION=2.5.0
BOOST_VERSION=1.60

AC_PROG_CXX
AC_GNU_SOURCE
AX_CXX_COMPILE_STDCXX_17(noext,mandatory)

AC_ARG_ENABLE(debug,[  --enable-debug    Turn on debugging],[
        case "${enableval}" in
           yes) debug=true ;
                DEBUG_CFLAGS="-DDEBUG -g" ;
                OPTIMIZE_CFLAGS="" ;;
            no) debug=false ;
                DEBUG_CFLAGS="-DNDEBUG" ;;
             *) AC_MSG_ERROR(bad value ${enableval} for --enable-debug) ;;
        esac
],[     debug=false
        DEBUG_CFLAGS="-DNDEBUG"
])
AM_CONDITIONAL(DEBUG, test x$debug = xtrue)

NP_APPS="niepce"
NP_APP_NIEPCE=yes

CPPFLAGS="$CPPFLAGS $DEBUG_CFLAGS -Wall -Wcast-align -Wcast-qual -Wpointer-arith -Wreturn-type"
CFLAGS="$CFLAGS $DEBUG_CFLAGS $OPTIMIZE_CFLAGS"
CXXFLAGS="$CXXFLAGS $DEBUG_CFLAGS $OPTIMIZE_CFLAGS"
dnl CFLAGS=""
LDFLAGS="$LDFLAGS"


dnl AC_PROG_INSTALL
AC_PROG_LIBTOOL

AC_LANG_CPLUSPLUS
AC_LANG_COMPILER_REQUIRE

dnl Framework requirements.
PKG_CHECK_MODULES(FRAMEWORK, [glibmm-2.68 >= $LIBGLIBMM_VERSION giomm-2.68 >= $LIBGIOMM_VERSION gthread-2.0 gtkmm-4.0 >= $LIBGTKMM_VERSION sqlite3 >= $SQLITE_VERSION exempi-2.0 >= $EXEMPI_VERSION libxml-2.0 >= $LIBXML2_VERSION shumate-1.0 >= 1.0.0])

dnl optional framework
PKG_CHECK_MODULES(GPHOTO, [libgphoto2 >= $LIBGPHOTO_VERSION libgphoto2_port])

dnl niepce
PKG_CHECK_MODULES(BABL, babl)
PKG_CHECK_MODULES(OPENRAW, libopenraw-0.3 >= $LIBOPENRAW_VERSION)
AC_SUBST(OPENRAW_CFLAGS)
AC_SUBST(OPENRAW_LIBS)

PKG_CHECK_MODULES(GEGL, gegl-0.4 >= $GEGL_VERSION)
AC_SUBST(GEGL_CFLAGS)
AC_SUBST(GEGL_LIBS)

dnl gexiv2 is needed in the Rust code
PKG_CHECK_MODULES(GEXIV2, gexiv2)
AC_SUBST(GEXIV2_LIBS)

AX_BOOST_BASE([$BOOST_VERSION])

dnl Relative googletest dir from top srcdir.
GTEST_ROOT=third_party/googletest/googletest
AC_SUBST(GTEST_ROOT)

dnl ***************************************************************
dnl Excessive warnings
dnl ***************************************************************
AX_CHECK_COMPILE_FLAG([-Werror=unknown-warning-option], [
	ax_compiler_flags_test="-Werror=unknown-warning-option"
], [
	ax_compiler_flags_test=""
])
AX_APPEND_COMPILE_FLAGS([ \
   -Wchar-subscripts \
   -Wcast-align \
   -Wclass-memaccess \
dnl sigc++ 2.10.0 breaks with that warning.
   -Wno-cast-function-type \
   -Wextra \
   -Wheader-guard \
   -Wformat \
   -Wformat-nonliteral \
   -Wformat-security \
   -Wlogical-not-parentheses \
   -Wmisleading-indentation \
   -Wmissing-noreturn \
   -Wnon-virtual-dtor \
   -Woverloaded-virtual \
dnl Gtk+ 3.22.30 breaks with that warning
   -Wno-parentheses \
   -Wpointer-arith \
dnl   -Wredundant-decls \
   -Wpointer-bool-conversion \
   -Wshadow \
   -Wsign-compare \
   -Wsuggest-override \
   -Wunreachable-code \
   -Wunused \
   -Wwrite-strings \
], [], [$ax_compiler_flags_test])

AC_ARG_ENABLE(asan,[  --enable-asan    Turn on address sanitizer],[
    AX_APPEND_COMPILE_FLAGS([-fsanitize=address -fno-omit-frame-pointer])
]
,
[

])

AC_CHECK_PROG(CARGO, [cargo], [yes], [no])
AS_IF(test x$CARGO = xno,
    AC_MSG_ERROR([cargo is required])
)
AC_CHECK_PROG(RUSTC, [rustc], [yes], [no])
AS_IF(test x$RUSTC = xno,
    AC_MSG_ERROR([rustc is required])
)

dnl Release build we do.
if test x$debug = xtrue; then
CARGO_TARGET_SUBDIR=debug
else
CARGO_TARGET_SUBDIR=release
fi
AC_SUBST(CARGO_TARGET_SUBDIR)


AC_CONFIG_FILES([
Makefile
art/Makefile
doc/Doxyfile
doc/Makefile
doc/doxygen/Makefile
data/Makefile
data/themes/Makefile
third_party/Makefile
src/Makefile
src/ext/Makefile
src/fwk/Makefile
src/fwk/base/Makefile
src/fwk/utils/Makefile
src/fwk/toolkit/Makefile
src/engine/Makefile
src/ncr/Makefile
po/Makefile.in
])

dnl =============
dnl niepce
dnl =============
AM_CONDITIONAL(NIEPCE_BUILD, test "$NP_APP_NIEPCE" = "yes")
if test "$NP_APP_NIEPCE" = "yes" ; then

MAJOR_VERSION=0
MINOR_VERSION=0
MICRO_VERSION=1

NIEPCE_VERSION="$MAJOR_VERSION.$MINOR_VERSION.$MICRO_VERSION"
AC_SUBST(NIEPCE_VERSION)

GETTEXT_PACKAGE=niepce
AC_SUBST(GETTEXT_PACKAGE)
AC_DEFINE_UNQUOTED([GETTEXT_PACKAGE], ["$GETTEXT_PACKAGE"],
                   [The domain to use with gettext])
AM_GNU_GETTEXT_VERSION([0.19.8])
AM_GNU_GETTEXT_REQUIRE_VERSION([0.19.8])
AM_GNU_GETTEXT([external])

NIEPCE_LOCALEDIR=[${datadir}/locale]
AC_SUBST(NIEPCE_LOCALEDIR)

AC_DEFINE_UNQUOTED([NIEPCE_BUILD_CONFIG], ["$NIEPCE_BUILD_CONFIG"], [The string used to hardcode the build config.])

AC_CONFIG_FILES([
src/niepce/ui/Makefile
src/niepce/modules/Makefile
src/niepce/modules/darkroom/Makefile
src/niepce/modules/interfaces/Makefile
src/niepce/modules/map/Makefile
src/niepce/Makefile
])

fi

AC_OUTPUT

echo "

  Build apps: ${NP_APPS}

"
