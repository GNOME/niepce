

AM_CPPFLAGS = -I$(top_srcdir)/src -I$(top_builddir)/src @FRAMEWORK_CFLAGS@

noinst_LIBRARIES = liblibraryclient.a

TEST_LIBS =  \
	liblibraryclient.a \
	$(top_builddir)/src/engine/libniepceengine.a \
	$(top_builddir)/target/debug/libniepce_rust.a \
	$(top_builddir)/src/engine/libniepceengineglue.a \
        $(top_builddir)/src/fwk/utils/libniepceutils.a \
	$(top_builddir)/src/fwk/toolkit/libniepceframework.a \
	$(top_builddir)/src/fwk/base/libfwkbase.a \
        @BOOST_UNIT_TEST_FRAMEWORK_LIBS@ \
	@FRAMEWORK_LIBS@ \
	@OPENRAW_LIBS@ \
	-ldl \
	$(NULL)

liblibraryclient_a_SOURCES = libraryclient.hpp  libraryclient.cpp \
	$(NULL)
