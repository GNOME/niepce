
AM_CPPFLAGS = -I$(top_srcdir)/src/ -I$(top_builddir)/src  @FRAMEWORK_CFLAGS@ \
	@GPHOTO_CFLAGS@ \
	$(NULL)

TEST_LIBS =  \
	libniepceengine.a \
	$(top_builddir)/target/debug/libniepce_rust.a \
	libniepceengineglue.a \
	$(top_builddir)/src/fwk/utils/libniepceutils.a \
	$(top_builddir)/src/fwk/toolkit/libniepceframework.a \
	$(top_builddir)/src/fwk/base/libfwkbase.a \
	@BOOST_UNIT_TEST_FRAMEWORK_LIBS@ \
	@FRAMEWORK_LIBS@ \
	@OPENRAW_LIBS@ \
	-ldl \
	$(NULL)

noinst_LIBRARIES = libniepceengine.a

libniepceengine_a_SOURCES = \
	db/libfile.hpp \
	db/label.hpp \
	db/storage.hpp \
	db/fsfile.hpp \
	db/metadata.hpp \
	db/properties.hpp db/properties.cpp \
	db/properties-def.hpp \
	library/clienttypes.hpp \
	library/thumbnailnotification.hpp \
	importer/iimporter.hpp \
	importer/directoryimporter.hpp \
	importer/directoryimporter.cpp \
	importer/cameraimporter.hpp \
	importer/cameraimporter.cpp \
	importer/importedfile.hpp \
	importer/importedfile.cpp \
	$(NULL)

