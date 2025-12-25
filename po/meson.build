find_program('xgettext', version: '>= 0.24')

i18n.gettext(
  gettext_package,
  args: [
    '--keyword=i18n',
    '--keyword=i18n_format!',
    '--keyword=i18n_nformat!:1,2'
  ],
  preset: 'glib'
)
