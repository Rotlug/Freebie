flatpak-builder --user --force-clean builddir com.github.rotlug.Freebie.json

meson setup builddir
ninja -C builddir

flatpak-builder --run builddir com.github.rotlug.Freebie.json freebie
