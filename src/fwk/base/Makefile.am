
niepce_cppflags = -I$(top_srcdir)/src \
	@BOOST_CPPFLAGS@ \
	@FRAMEWORK_CFLAGS@ \
	$(NULL)

AM_CPPFLAGS = $(niepce_cppflags)

noinst_LIBRARIES = libfwkbase.a

TESTS = testmoniker testgeometry testmap\
	testoption

check_PROGRAMS = testmoniker testgeometry testmap\
	testoption


testing_cppflags = -I@top_srcdir@/@GTEST_ROOT@/include $(niepce_cppflags)
testing_ldadd = @top_builddir@/third_party/libgtest.a libfwkbase.a \
	@FRAMEWORK_LIBS@

testmoniker_SOURCES = t/testmoniker.cpp
testmoniker_CPPFLAGS = $(testing_cppflags)
testmoniker_LDADD = $(testing_ldadd)

testgeometry_SOURCES = t/testgeometry.cpp
testgeometry_CPPFLAGS = $(testing_cppflags)
testgeometry_LDADD = $(testing_ldadd)

testmap_SOURCES = t/testmap.cpp
testmap_CPPFLAGS = $(testing_cppflags)
testmap_LDADD = $(testing_ldadd)

testoption_SOURCES = t/testoption.cpp
testoption_CPPFLAGS = $(testing_cppflags)
testoption_LDADD = $(testing_ldadd)

libfwkbase_a_SOURCES = colour.hpp colour.cpp \
	autoflag.hpp \
	date.hpp date.cpp \
	debug.hpp debug.cpp \
	moniker.hpp moniker.cpp \
	geometry.hpp geometry.cpp \
	singleton.hpp \
	string.hpp string.cpp \
	util.hpp \
	map.hpp \
	option.hpp \
	propertybag.hpp propertybag.cpp \
	$(NULL)
