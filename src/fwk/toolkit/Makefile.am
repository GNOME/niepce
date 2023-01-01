

niepce_cppflags = -I$(top_srcdir)/src -I$(top_builddir)/src -I$(top_srcdir)/src/ext \
	-DDATADIR=\"$(datadir)\" \
	@FRAMEWORK_CFLAGS@ \
	@GPHOTO_CFLAGS@ \
	@OPENRAW_CFLAGS@ \
	$(NULL)

rust_ldadd = $(top_builddir)/target/@CARGO_TARGET_SUBDIR@/libniepce_rust.a \
	@OPENRAW_LIBS@ \
	@GEXIV2_LIBS@ \
	$(NULL)

AM_CPPFLAGS = $(niepce_cppflags)

noinst_LIBRARIES = libniepceframework.a

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
testconfigdatabinder_LDADD = $(testing_ldadd) \
	$(rust_ldadd) \
	$(NULL)

libniepceframework_a_SOURCES = \
	../cxx_colour_bindings.cpp \
	../cxx_fwk_bindings.cpp \
	../cxx_eng_bindings.cpp \
	../cxx_widgets_bindings.cpp \
	../cxx_npc_bindings.cpp \
	application.hpp application.cpp \
	appframe.hpp appframe.cpp \
	dialog.hpp dialog.cpp \
	frame.hpp frame.cpp \
	controller.hpp controller.cpp \
	uicontroller.hpp uicontroller.cpp \
	uiresult.hpp uiresult.cpp \
	mapcontroller.hpp mapcontroller.cpp \
	notification.hpp \
	mimetype.hpp mimetype.cpp \
	imageloader.hpp imageloader.cpp \
	notificationcenter.hpp notificationcenter.cpp \
	configdatabinder.hpp configdatabinder.cpp \
	gdkutils.hpp gdkutils.cpp \
	gtkutils.hpp gtkutils.cpp \
	widgets/addinstreemodel.hpp widgets/addinstreemodel.cpp \
	widgets/toolboxitemwidget.hpp widgets/toolboxitemwidget.cpp \
	widgets/editablehscale.hpp widgets/editablehscale.cpp \
	widgets/dock.cpp widgets/dock.hpp \
	dockable.hpp dockable.cpp \
	undo.hpp \
	command.hpp \
	$(NULL)
