# `hcdl` Changelog

## v0.10.1

  - Add support for [`NO_COLOR`] environment variable

## v0.10.0

  - Default to embedding the GPG key in the application for ease of use when
    installing via Cargo
    - Install with `cargo install --no-default-features hcdl` or compile with
      `cargo build --no-default-features hcdl` to avoid this, as noted in the
      `README.md`

## v0.9.3

  - Add CRC32 verification of files extracted from zip
  - Extraction of files from zips now go via temporary files to avoid
    clobbering existing working binaries in the event of extraction failure.
  - On `unix` type systems the permissions of the extracted files are now taken
    from the zipfile instead of being forced to `0755`

## v0.9.2

  - Improve output when unzipping files
  - Use safer unzipping method which properly sanitizes filenames

## v0.9.1

  - Add missing message for `--download-only` mode
  - Ensure that files under `test-data` and `gpg` directories retain their
    line-endings on Windows
  - Fix issues with the `--install-dir` option
  - Use `OsStr` based validator for `--install-dir`

## v0.9.0

  - Initial release

  <!-- links -->
  [`NO_COLOR`]: https://no-color.org/
