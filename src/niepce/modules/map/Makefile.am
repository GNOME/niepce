

AM_CPPFLAGS = -I$(top_srcdir)/src -I$(top_builddir)/src -I$(top_srcdir)/src/niepce \
	-I$(top_srcdir)/src/ext \
	-DDATADIR=\"$(datadir)\" \
	@FRAMEWORK_CFLAGS@

noinst_LIBRARIES=libmodulemap.a


libmodulemap_a_SOURCES = \
	mapmodule.hpp mapmodule.cpp \
	$(NULL)
