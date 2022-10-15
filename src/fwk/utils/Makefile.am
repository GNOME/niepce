
niepce_cppflags = -I$(top_srcdir)/src/ -I$(top_builddir)/src \
	@FRAMEWORK_CFLAGS@

AM_CPPFLAGS = $(niepce_cppflags)

TESTS = testpathutils \
	teststringutils

EXTRA_DIST = test.xmp test2.ufraw MODULES_HOWTO

check_PROGRAMS = testpathutils \
	teststringutils


testing_cppflags = -I@top_srcdir@/@GTEST_ROOT@/include $(niepce_cppflags)
testing_ldadd = @top_builddir@/third_party/libgtest.a \
	libniepceutils.a \
	../base/libfwkbase.a \
	@FRAMEWORK_LIBS@

teststringutils_SOURCES = t/teststringutils.cpp
teststringutils_CPPFLAGS = $(testing_cppflags)
teststringutils_LDADD = $(testing_ldadd)

testpathutils_SOURCES = t/testpathutils.cpp
testpathutils_CPPFLAGS = $(testing_cppflags)
testpathutils_LDADD = $(testing_ldadd)

noinst_LIBRARIES = libniepceutils.a

libniepceutils_a_SOURCES = \
	init.hpp init.cpp \
	exception.hpp exception.cpp \
	fsutils.cpp fsutils.hpp \
	mtqueue.hpp stringutils.hpp \
	exempi.hpp exempi.cpp \
	buffer.hpp logstreamutils.hpp \
	boost.hpp \
	files.hpp files.cpp \
	thread.hpp thread.cpp worker.hpp \
	pathutils.hpp pathutils.cpp \
	databinder.hpp databinder.cpp \
	dynamicmodule.hpp dynamicmodule.cpp \
	modulefactory.hpp \
	modulemanager.hpp modulemanager.cpp \
	$(NULL)

