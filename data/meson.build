blueprints = custom_target('blueprints',
  input: files(
    'gtk/window.blp',
    'gtk/main_page.blp',
    'gtk/game_page.blp',
    'gtk/game.blp',
    'gtk/browse_view.blp',
    'gtk/play_view.blp',
    'gtk/proton_page.blp',
    'gtk/igdb_page.blp',
    'gtk/preferences.blp'
  ),
  output: '.',
  command: [find_program('blueprint-compiler'), 'batch-compile', '@OUTPUT@', '@CURRENT_SOURCE_DIR@', '@INPUT@'],
)

gnome.compile_resources('freebie',
  'freebie.gresource.xml',
  gresource_bundle: true,
  install: true,
  install_dir: pkgdatadir,
  dependencies: blueprints
)

desktop_file = i18n.merge_file(
        input: 'com.github.rotlug.Freebie.desktop.in',
       output: 'com.github.rotlug.Freebie.desktop',
         type: 'desktop',
       po_dir: '../po',
      install: true,
  install_dir: get_option('datadir') / 'applications'
)

desktop_utils = find_program('desktop-file-validate', required: false)
if desktop_utils.found()
  test('Validate desktop file', desktop_utils, args: [desktop_file])
endif

appstream_file = i18n.merge_file(
        input: 'com.github.rotlug.Freebie.metainfo.xml.in',
       output: 'com.github.rotlug.Freebie.metainfo.xml',
       po_dir: '../po',
      install: true,
  install_dir: get_option('datadir') / 'metainfo'
)

appstreamcli = find_program('appstreamcli', required: false, disabler: true)
test('Validate appstream file', appstreamcli,
     args: ['validate', '--no-net', '--explain', appstream_file])

install_data('com.github.rotlug.Freebie.gschema.xml',
  install_dir: get_option('datadir') / 'glib-2.0' / 'schemas'
)

compile_schemas = find_program('glib-compile-schemas', required: false, disabler: true)
test('Validate schema file',
     compile_schemas,
     args: ['--strict', '--dry-run', meson.current_source_dir()])

subdir('icons')
