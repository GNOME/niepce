SUBDIRS = themes

desktop_in_files = org.gnome.Niepce.desktop.in
desktop_files    = $(desktop_in_files:.desktop.in=.desktop)

%.desktop: %.desktop.in
	$(AM_V_GEN)$(MSGFMT) --desktop --template $< -d $(top_srcdir)/po -o $@

desktopdir   = $(datadir)/applications
desktop_DATA = $(desktop_files)

appdatadir = $(datarootdir)/appdata
appdata_in_files = org.gnome.Niepce.appdata.xml.in
appdata_files = $(appdata_in_files:.xml.in=.xml)
appdata_DATA  = $(appdata_files)

%.appdata.xml: %.appdata.xml.in
	$(AM_V_GEN)$(MSGFMT) --xml --template $< -d $(top_srcdir)/po -o $@

icon_files = icons/niepce-jpg-fmt.png \
	icons/niepce-raw-fmt.png \
	icons/niepce-rawjpeg-fmt.png \
	icons/niepce-tiff-fmt.png \
	icons/niepce-png-fmt.png \
	icons/niepce-img-fmt.png \
	icons/niepce-unknown-fmt.png \
	icons/niepce-video-fmt.png \
	icons/niepce-set-star.png \
	icons/niepce-unset-star.png \
	icons/niepce-flag-pick.png \
	icons/niepce-flag-reject.png \
	icons/niepce-transform-rotate.png \
	icons/niepce-image-generic.png \
	icons/niepce-image-missing.png \
	icons/niepce-missing.png \
	icons/open-book-symbolic.svg \
	$(null)

EXTRA_DIST = \
        $(desktop_in_files) \
	$(appdata_in_files) \
	$(icon_files) \
	$(null)


CLEANFILES = \
	$(desktop_files)                        \
	$(appdata_DATA)	                        \
	$(null)

DISTCLEANFILES =                                \
	$(null)

check:
	desktop-file-validate $(desktop_files)
