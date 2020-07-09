# `hcdl` Changelog

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
