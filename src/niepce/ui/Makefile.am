
AM_CPPFLAGS = -I$(top_srcdir)/src -I$(top_builddir)/src -I$(srcdir)/.. \
	-I$(top_srcdir)/src/ext \
	-DDATADIR=\"$(datadir)\" \
	@FRAMEWORK_CFLAGS@ @GPHOTO_CFLAGS@ \
	$(NULL)

noinst_LIBRARIES = libniepceui.a

PUBLICHEADERS = ilibrarymodule.hpp dialogs/importers/iimporterui.hpp

libniepceui_a_SOURCES = \
	niepcewindow.hpp \
	niepceapplication.hpp niepceapplication.cpp \
	gridviewmodule.hpp gridviewmodule.cpp \
	metadatapanecontroller.hpp metadatapanecontroller.cpp \
	dialogs/editlabels.hpp dialogs/editlabels.cpp \
	dialogs/preferencesdialog.hpp dialogs/preferencesdialog.cpp \
	selectioncontroller.hpp \
	filmstripcontroller.hpp \
	$(PUBLICHEADERS) \
	$(NULL)
