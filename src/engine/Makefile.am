
AM_CPPFLAGS = -I$(top_srcdir)/src/ -I$(top_builddir)/src  @FRAMEWORK_CFLAGS@ \
	@GPHOTO_CFLAGS@ \
	$(NULL)

noinst_LIBRARIES = libniepceengine.a

libniepceengine_a_SOURCES = \
	db/libfile.hpp \
	db/label.hpp \
	db/storage.hpp \
	db/fsfile.hpp \
	db/metadata.hpp \
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

