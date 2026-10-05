grainx (grainx)
===============

Shows live computer and process activity in a terminal. This archive holds the `grainx` command-line tool, built from the tagged release.

Install
-------
Put the `grainx` binary (`grainx.exe` on Windows) in a directory on your PATH,
or use the installer, which does that for you and checks the checksum:

  macOS / Linux:  curl -fsSL https://raw.githubusercontent.com/rustfuture/grainx/main/install.sh | sh
  Windows:        irm https://raw.githubusercontent.com/rustfuture/grainx/main/install.ps1 | iex

Then:

  grainx --version
  grainx --help

Verify this download
--------------------
Each archive has a matching .sha256 file on the release page:

  sha256sum -c grainx-<version>-<target>.tar.gz.sha256      (Linux)
  shasum -a 256 -c grainx-<version>-<target>.tar.gz.sha256  (macOS)

Docs and source: https://github.com/rustfuture/grainx
License: see LICENSE
