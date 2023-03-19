# `hcdl` Changelog

## v0.13.0

  - Bump MSRV to 1.64.0
  - Bump dependencies

## v0.12.0

  - Bump MSRV to 1.56.1
  - Bump dependencies
  - Replaced uses of [lazy-static] with [once_cell]
  - Split progress bar code out into its own file
  - Move from [gpgrv] crate to [pgp] crate to fix signature validation
  - Update to use the new [Hashicorp Releases API]

## v0.11.0

  - Fix macOS conditional compilation
  - Update to reqwest 0.11.0
  - Update to tokio 1.0
  - Bump MSRV to 1.46.0
  - Update signature after [HCSEC-2021-12]

## v0.10.2

  - Add support for generating shell tab completions
    - Support included by default under the `shell_completion` feature
    - Supported shells are currently: Bash, Elvish, Fish, PowerShell, and ZSH
      as listed in the [Clap Shell enum]
  - Reduced the frequency of progress bar updates
  - Fixed the `NO_COLOR` progress template, which could have resulted in colour
    in `NO_COLOR` mode
  - Updated to [gpgrv] `0.3.0` and simplified some error handling logic in
    gpgrv related tasks
    - The changelog for this version isn't in the gpgrv GitHub repository. A
      manual comparison of the differences was performed between crates `0.2.3`
      and `0.3.0`, which were obtained from [crates.io]

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
[clap]: https://crates.io/crates/clap
[crates.io]: https://crates.io/
[gpgrv]: https://crates.io/crates/gpgrv
[lazy-static]: https://crates.io/crates/lazy-static
[once_cell]: https://crates.io/crates/once_cell
[pgp]: https://crates.io/crates/pgp
[Clap Shell enum]: https://docs.rs/clap/2.33.3/clap/enum.Shell.html#variants
[Hashicorp Releases API]: https://www.hashicorp.com/blog/announcing-the-hashicorp-releases-api
[HCSEC-2021-12]: https://discuss.hashicorp.com/t/hcsec-2021-12-codecov-security-event-and-hashicorp-gpg-key-exposure/23512
