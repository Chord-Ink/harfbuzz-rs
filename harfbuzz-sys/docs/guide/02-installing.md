# Installing HarfBuzz

*Adapted from the HarfBuzz 14.3.0 documentation, which is licensed under the terms in the HarfBuzz COPYING file.*

## Downloading HarfBuzz

The HarfBuzz source code is hosted at [github.com/harfbuzz/harfbuzz](https://github.com/harfbuzz/harfbuzz).

Tarball releases and Win32 binary bundles (which include the libharfbuzz DLL, hb-view.exe, hb-shape.exe, and all dependencies) of HarfBuzz can be downloaded from [github.com/harfbuzz/harfbuzz/releases](https://github.com/harfbuzz/harfbuzz/releases).

Release notes are posted with each new release to provide an overview of the changes. The project [tracks bug reports and other issues](https://github.com/harfbuzz/harfbuzz/issues) on GitHub. Discussion and questions are welcome on [GitHub](https://github.com/harfbuzz/harfbuzz/discussions) as well.

The API included in the `hb.h` file will not change in a compatibility-breaking way in any release. However, other, peripheral headers are more likely to go through minor modifications. We will do our best to never change APIs in an incompatible way. We will *never* break the ABI.

## Building HarfBuzz

### Building on Linux

*(1)* To build HarfBuzz on Linux, you must first install the development packages for FreeType, Cairo, and GLib. The exact commands required for this step will vary depending on the Linux distribution you use.

For example, on an Ubuntu or Debian system, you would run:

```sh
sudo apt install gcc g++ libfreetype6-dev libglib2.0-dev libcairo2-dev
```

On Fedora, RHEL, CentOS, or other Red-Hat–based systems, you would run:

```sh
sudo yum install gcc gcc-c++ freetype-devel glib2-devel cairo-devel
```

*(2)* The next step depends on whether you are building from the source in a downloaded release tarball or from the source directly from the git repository.

*(2)(a)* If you downloaded the HarfBuzz source code in a tarball, you can now extract the source.

From a shell in the top-level directory of the extracted source code, you can run `meson build` followed by `meson compile -C build` as with any other standard package.

This should leave you with a shared library in the `src/` directory, and a few utility programs including `hb-view` and `hb-shape` under the `util/` directory.

*(2)(b)* If you are building from the source in the HarfBuzz git repository, rather than installing from a downloaded tarball release, then you must install two more auxiliary tools before you can build for the first time: `pkg-config`.

On Ubuntu or Debian, run:

```sh
sudo apt-get install meson pkg-config gtk-doc-tools
```

On Fedora, RHEL, CentOS, run:

```sh
sudo yum install meson pkgconfig gtk-doc
```

With `pkg-config` installed, you can now run `meson build` then `meson compile -C build` to build HarfBuzz.

### Building on Windows

[Install meson](https://mesonbuild.com/Getting-meson.html) and run (from the console) `meson build` (by default bundled dependencies are not built, `--wrap-mode=default` overrides this), then `meson compile -C build` to build HarfBuzz.

### Building on macOS

There are two ways to build HarfBuzz on Mac systems: MacPorts and Homebrew. The process is similar to the process used on a Linux system.

*(1)* You must first install the development packages for FreeType, Cairo, and GLib. If you are using MacPorts, you should run:

```sh
sudo port install freetype glib2 cairo
```

If you are using Homebrew, you should run:

```sh
brew install freetype glib cairo
```

*(2)* The next step depends on whether you are building from the source in a downloaded release tarball or from the source directly from the git repository.

*(2)(a)* If you are installing HarfBuzz from a downloaded tarball release, extract the tarball and open a Terminal in the extracted source-code directory. Run:

```sh
meson build
```

followed by:

```sh
meson compile -C build
```

to build HarfBuzz.

*(2)(b)* Alternatively, if you are building HarfBuzz from the source in the HarfBuzz git repository, then you must install several built-time dependencies before proceeding.

If you are using MacPorts, you should run:

```sh
sudo port install meson pkgconfig gtk-doc
```

to install the build dependencies.

If you are using Homebrew, you should run:

```sh
brew install meson pkgconfig gtk-doc
```

Finally, you can run:

```sh
meson build
```

*(3)* You can now build HarfBuzz (on either a MacPorts or a Homebrew system) by running:

```sh
meson build
```

followed by:

```sh
meson compile -C build
```

This should leave you with a shared library in the `src/` directory, and a few utility programs including `hb-view` and `hb-shape` under the `util/` directory.

### Configuration options

The instructions in the "Building HarfBuzz" section will build the source code under its default configuration. If needed, the following additional configuration options are available.

**`-Dglib=enabled`**

Use [GLib](https://developer.gnome.org/glib/). *(Default = auto)*

This option enables or disables usage of the GLib library. The default setting is to check for the presence of GLib and, if it is found, build with GLib support. GLib is native to GNU/Linux systems but is available on other operating system as well.

**`-Dgobject=enabled`**

Use [GObject](https://developer.gnome.org/gobject/stable/). *(Default = no)*

This option enables or disables usage of the GObject library. The default setting is to check for the presence of GObject and, if it is found, build with GObject support. GObject is native to GNU/Linux systems but is available on other operating system as well.

**`-Dcairo=enabled`**

Use [Cairo](https://cairographics.org/). *(Default = auto)*

This option enables or disables usage of the Cairo graphics-rendering library. The default setting is to check for the presence of Cairo and, if it is found, build with Cairo support.

Note: Cairo is used only by the HarfBuzz command-line utilities, and not by the HarfBuzz library.

**`-Dicu=enabled`**

Use the [ICU](http://site.icu-project.org/home) library. *(Default = auto)*

This option enables or disables usage of the *International Components for Unicode* (ICU) library, which provides access to Unicode Character Database (UCD) properties as well as normalization and conversion functions. The default setting is to check for the presence of ICU and, if it is found, build with ICU support.

**`-Dgraphite=enabled`**

Use the [Graphite2](http://graphite.sil.org/) library. *(Default = no)*

This option enables or disables usage of the Graphite2 library, which provides support for the Graphite shaping model.

**`-Dfreetype=enabled`**

Use the [FreeType](https://www.freetype.org/) library. *(Default = auto)*

This option enables or disables usage of the FreeType font-rendering library. The default setting is to check for the presence of FreeType and, if it is found, build with FreeType support.

**`-Dgdi=enabled`**

Use the [Uniscribe](https://docs.microsoft.com/en-us/windows/desktop/intl/uniscribe) library (experimental). *(Default = no)*

This option enables or disables usage of the Uniscribe font-rendering library. Uniscribe is available on Windows systems. Uniscribe support is used only for testing purposes and does not need to be enabled for HarfBuzz to run on Windows systems.

**`-Ddirectwrite=enabled`**

Use the [DirectWrite](https://docs.microsoft.com/en-us/windows/desktop/directwrite/direct-write-portal) library (experimental). *(Default = no)*

This option enables or disables usage of the DirectWrite font-rendering library. DirectWrite is available on Windows systems. DirectWrite support is used only for testing purposes and does not need to be enabled for HarfBuzz to run on Windows systems.

**`-Dcoretext=enabled`**

Use the [CoreText](https://developer.apple.com/documentation/coretext) library. *(Default = no)*

This option enables or disables usage of the CoreText library. CoreText is available on macOS and iOS systems.

**`-Ddocs=enabled`**

Use [GTK-Doc](https://github.com/GNOME/gtk-doc). *(Default = no)*

This option enables the building of the documentation.

### Single-file build with `harfbuzz-world.cc`

For projects that want HarfBuzz embedded without any build-system glue, the source ships a single-file amalgamation at `src/harfbuzz-world.cc` that `#include`s every `.cc` in the library so the whole thing compiles as one translation unit. Pair it with the `HB_TINY` / `HB_HAS_*` compile-time switches (see the Configuration options section above) to pick which subsystems get pulled in.

A live in-browser playground built against this exact file lives at [harfbuzz-world.cc](https://harfbuzz-world.cc/); its [source](https://github.com/harfbuzz/harfbuzz-world.cc) is a worked example of dropping HarfBuzz into a project.
