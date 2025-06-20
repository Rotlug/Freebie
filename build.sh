meson setup builddir --prefix=$PWD/_build
meson compile -C builddir
meson install -C builddir

cd _build
./bin/freebie
