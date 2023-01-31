
niepce_cppflags = -I$(top_srcdir)/src/ -I$(top_builddir)/src \
	@FRAMEWORK_CFLAGS@

AM_CPPFLAGS = $(niepce_cppflags)

TESTS = testpathutils

EXTRA_DIST = test.xmp test2.ufraw MODULES_HOWTO

check_PROGRAMS = testpathutils


testing_cppflags = -I@top_srcdir@/@GTEST_ROOT@/include $(niepce_cppflags)
testing_ldadd = @top_builddir@/third_party/libgtest.a \
	libniepceutils.a \
	../base/libfwkbase.a \
	@FRAMEWORK_LIBS@

testpathutils_SOURCES = t/testpathutils.cpp
testpathutils_CPPFLAGS = $(testing_cppflags)
testpathutils_LDADD = $(testing_ldadd)

noinst_LIBRARIES = libniepceutils.a

libniepceutils_a_SOURCES = \
	init.hpp init.cpp \
	exempi.hpp exempi.cpp \
	pathutils.hpp pathutils.cpp \
	databinder.hpp databinder.cpp \
	$(NULL)

