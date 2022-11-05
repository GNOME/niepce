

AM_CPPFLAGS = -I$(top_srcdir)/src -I$(top_builddir)/src -I$(top_srcdir)/src/niepce \
	-I$(top_srcdir)/src/ext \
	-DDATADIR=\"$(datadir)\" \
	@FRAMEWORK_CFLAGS@ \
	@GEGL_CFLAGS@ @OPENRAW_CFLAGS@


noinst_LIBRARIES=libmoduledarkroom.a


libmoduledarkroom_a_SOURCES = darkroommodule.cpp darkroommodule.hpp \
	imagecanvas.hpp imagecanvas.cpp \
	toolboxcontroller.hpp toolboxcontroller.cpp \
	dritemwidget.hpp dritemwidget.cpp \
	$(NULL)
