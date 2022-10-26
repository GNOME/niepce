
AM_CPPFLAGS = -I$(top_srcdir)/src -I$(top_builddir)/src -I$(srcdir)/.. \
	-I$(top_srcdir)/src/ext \
	-DDATADIR=\"$(datadir)\" \
	@FRAMEWORK_CFLAGS@ @GPHOTO_CFLAGS@ \
	$(NULL)

noinst_LIBRARIES = libniepceui.a

PUBLICHEADERS = ilibrarymodule.hpp imoduleshell.hpp dialogs/importers/iimporterui.hpp

libniepceui_a_SOURCES = \
	niepcewindow.hpp \
	niepceapplication.hpp niepceapplication.cpp \
	gridviewmodule.hpp gridviewmodule.cpp \
	moduleshell.hpp moduleshell.cpp \
	moduleshellwidget.hpp moduleshellwidget.cpp \
	imageliststore.hpp imageliststore.cpp\
	metadatapanecontroller.hpp metadatapanecontroller.cpp \
	dialogs/editlabels.hpp dialogs/editlabels.cpp \
	dialogs/importdialog.hpp dialogs/importdialog.cpp \
	dialogs/preferencesdialog.hpp dialogs/preferencesdialog.cpp \
	dialogs/importdialog.hpp dialogs/importdialog.cpp \
	dialogs/importers/importerui.hpp \
	dialogs/importers/importerui.cpp \
	dialogs/importers/directoryimporterui.hpp \
	dialogs/importers/directoryimporterui.cpp \
	dialogs/importers/cameraimporterui.hpp \
	dialogs/importers/cameraimporterui.cpp \
	selectioncontroller.hpp selectioncontroller.cpp \
	filmstripcontroller.hpp \
	$(PUBLICHEADERS) \
	$(NULL)
