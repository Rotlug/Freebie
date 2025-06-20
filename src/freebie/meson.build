pkgdatadir = get_option('prefix') / get_option('datadir') / meson.project_name()
moduledir = pkgdatadir / 'freebie'
gnome = import('gnome')

python = import('python')

conf = configuration_data()
conf.set('PYTHON', python.find_installation('python3').full_path())
conf.set('VERSION', meson.project_version())
conf.set('localedir', get_option('prefix') / get_option('localedir'))
conf.set('pkgdatadir', pkgdatadir)

configure_file(
  input: 'freebie.in',
  output: 'freebie',
  configuration: conf,
  install: true,
  install_dir: get_option('bindir'),
  install_mode: 'r-xr-xr-x'
)

freebie_sources = [
  '__init__.py',
  'main.py',
  'window.py',
  'game_manager.py',
  'game_box.py'
]

install_subdir('backend', install_dir: moduledir)
install_subdir('pages', install_dir: moduledir)
install_subdir('dialogs', install_dir: moduledir)
install_data(freebie_sources, install_dir: moduledir)
