
SUBDIRS = ui modules

AM_CPPFLAGS = -DDATADIR=\"$(datadir)\" \
	-I$(top_srcdir)/src/niepce -I$(top_srcdir)/src -I$(top_builddir)/src \
	@FRAMEWORK_CFLAGS@ \
	-DNIEPCE_LOCALEDIR=\"@NIEPCE_LOCALEDIR@\"

bin_PROGRAMS = niepce

niepce_LDFLAGS = -Wl,--start-group
niepce_LDADD = \
	$(top_builddir)/src/niepce/ui/libniepceui.a \
	$(top_builddir)/src/niepce/modules/darkroom/libmoduledarkroom.a \
	$(top_builddir)/src/niepce/modules/map/libmodulemap.a \
	$(top_builddir)/src/niepce/modules/interfaces/libinterfaces.a \
	$(top_builddir)/src/fwk/toolkit/libniepceframework.a \
	$(top_builddir)/src/fwk/base/libfwkbase.a \
	$(top_builddir)/src/ncr/libncr.a \
	$(top_builddir)/target/@CARGO_TARGET_SUBDIR@/libniepce_rust.a \
	$(top_builddir)/src/engine/libniepceengine.a \
	$(top_builddir)/src/fwk/utils/libniepceutils.a \
	@FRAMEWORK_LIBS@ \
	@GPHOTO_LIBS@ \
	@BABL_LIBS@ \
	@GEGL_LIBS@ @OPENRAW_LIBS@ \
	@GEXIV2_LIBS@ \
	-ldl \
	$(NULL)

gladefiles = ui/dialogs/preferences.ui \
	ui/dialogs/importdialog.ui \
	ui/dialogs/editlabels.ui \
	ui/dialogs/importers/directoryimporterui.ui \
	ui/dialogs/importers/cameraimporterui.ui \
	$(top_srcdir)/niepce-main/src/niepce/ui/dialogs/importlibrary.ui \
	$(top_srcdir)/niepce-main/src/niepce/ui/dialogs/importlibrary/lrimport_root_row.ui \
	$(NULL)

EXTRA_DIST = $(gladefiles)

## Resources compilation
npc-resources.c: gresource.xml $(gladefiles)
	$(AM_V_GEN) glib-compile-resources \
                --sourcedir=$(top_srcdir) --target=$@ \
                --generate-source --c-name npc $(srcdir)/gresource.xml

npc-resources.h: gresource.xml $(gladefiles)
	$(AM_V_GEN) glib-compile-resources                                  \
                --sourcedir=$(top_srcdir) --target=$@ \
		--generate-header --c-name npc $(srcdir)/gresource.xml

npc-resources.gresource: gresource.xml $(gladefiles)
	$(AM_V_GEN) glib-compile-resources                                  \
                --sourcedir=$(top_srcdir) --target=$@ \
		$(srcdir)/gresource.xml

BUILT_SOURCES = \
        npc-resources.c \
        npc-resources.h \
        npc-resources.gresource

niepce_SOURCES = \
	lnlistener.hpp \
	npc-resources.c \
	npc-resources.h \
	main.cpp
