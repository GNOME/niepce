

niepce_cppflags = -I$(top_srcdir)/src -I$(top_srcdir)/src/ext \
	-DDATADIR=\"$(datadir)\" \
	@FRAMEWORK_CFLAGS@ \
	@GPHOTO_CFLAGS@ \
	@OPENRAW_CFLAGS@ \
	$(NULL)

AM_CPPFLAGS = $(niepce_cppflags)

noinst_LIBRARIES = libniepceframework.a

noinst_HEADERS =   \
	  goocanvas_proxy_header.hpp

TESTS = testconfigdatabinder

check_PROGRAMS = testconfigdatabinder

testing_cppflags = -I@top_srcdir@/@GTEST_ROOT@/include $(niepce_cppflags)
testing_ldadd = @top_builddir@/third_party/libgtest.a \
	libniepceframework.a \
	../base/libfwkbase.a \
	../utils/libniepceutils.a \
        @FRAMEWORK_LIBS@

testconfigdatabinder_SOURCES = t/testconfigdatabinder.cpp
testconfigdatabinder_CPPFLAGS = $(testing_cppflags)
testconfigdatabinder_LDADD = $(testing_ldadd)

libniepceframework_a_SOURCES = configuration.hpp configuration.cpp \
	application.hpp application.cpp \
	appframe.hpp appframe.cpp \
	dialog.hpp dialog.cpp \
	frame.hpp frame.cpp \
	controller.hpp controller.cpp \
	uicontroller.hpp uicontroller.cpp \
	uiresult.hpp uiresult.cpp \
	thumbnail.hpp thumbnail.cpp \
	mapcontroller.hpp mapcontroller.cpp \
	notification.hpp \
	mimetype.hpp mimetype.cpp \
	imageloader.hpp imageloader.cpp \
	notificationcenter.hpp notificationcenter.cpp \
	configdatabinder.hpp configdatabinder.cpp \
	gdkutils.hpp gdkutils.cpp \
	gtkutils.hpp gtkutils.cpp \
	gphoto.hpp gphoto.cpp \
	widgets/addinstreemodel.hpp widgets/addinstreemodel.cpp \
	widgets/toolboxitemwidget.hpp widgets/toolboxitemwidget.cpp \
	widgets/editablehscale.hpp widgets/editablehscale.cpp \
	widgets/dock.cpp widgets/dock.hpp \
	widgets/notabtextview.hpp widgets/notabtextview.cpp \
	widgets/tokentextview.hpp widgets/tokentextview.cpp \
	dockable.hpp dockable.cpp \
	metadatawidget.hpp metadatawidget.cpp \
	undo.hpp undo.cpp \
	command.hpp \
	$(NULL)
