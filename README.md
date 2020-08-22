# `hcdl`: HashiCorp Downloader

`hcdl` is a tool for easily downloading and installing [HashiCorp] products.

## Installation

`hcdl` is available for install from [crates.io] if you have the stable [Rust]
toolchain of at least v1.40.0 installed.

This can be done with the standard Cargo install command:

```shell
$ cargo install hcdl
```

By default, `hcdl` will embed the HashiCorp GPG key into the binary to
facilitate ease of use when installing via Cargo.

The GPG key that will be used can be found in the `gpg` directory of the source
code in the `hashicorp.asc` file. You may verify this key against the details
given at https://www.hashicorp.com/security as detailed below.

If you do not trust the provided GPG key, you can disable the embedding when
installing with the following command:

```shell
$ cargo install --no-default-features hcdl
```

## Crate Features

`hcdl` includes two features:

  - `embed_gpg_key`, which embeds the HashiCorp GPG key required to verify the
    signature of downloaded files
  - `shell_completion`, which adds the `--completions` CLI argument to generate
    completions for your chosen shell

Both of these features are enabled by default. If you wished to install `hcdl`
without the embedded GPG key, but with shell completion generation support, you
could install as follows:

```shell
$ cargo install \
    --no-default-features \
    --features shell_completion \
    hcdl
```

## Usage

`hcdl` usage is very simple, for example, if we want to download and install
the latest version of [Terraform], we can run the following:

```shell
$ hcdl terraform
```

You'll see output like the following:

```
Latest version: terraform v0.12.28 from Thu, 25 Jun 2020 16:21:37 +0000
Downloading and verifying signature of terraform_0.12.28_SHA256SUMS...
Verified against terraform_0.12.28_SHA256SUMS.sig.
Downloading terraform_0.12.28_freebsd_amd64.zip...
  [00:00:04] [########################################] 27.07MB/27.07MB (0s) done.
SHA256 of terraform_0.12.28_freebsd_amd64.zip OK.
Unzipping contents of 'terraform_0.12.28_freebsd_amd64.zip' to '/home/user/.local/bin'
-> Extracting 'terraform' to '/home/user/.local/bin'...
Installation successful.
```

`hcdl` has performed the following steps:

  - Loaded the HashiCorp GPG key
  - Found the latest version of Terraform
  - Downloaded the SHA256SUMS GPG signature file
  - Downloaded the SHA256SUMS file
  - Verified the SHA256SUMS file against the signature
  - Downloaded the latest version of Terraform
  - Verified that the SHA256 of the downloaded file matches the SHA256SUMS file
  - Extracted the `terraform` binary to a temporary file
  - Ensured that the CRC32 of the extracted temporary file matches the record
    in the zip file
  - Installed the `terraform` binary to `~/.local/bin` by moving the temporary
    file to the appropriate location
  - Set the appropriate permissions on the extracted binary

By default, `hcdl` will download products for the operating system and
architecture that it was compiled for (above we were running `hcdl` on an
x86\_64 [FreeBSD] machine), however, you can download any product for any OS
and architecture you like by specifying the `--os` and `--arch` options.

## HashiCorp GPG Key

Due to the GPG signature checking, `hcdl` needs to know the HashiCorp GPG key.
There are two ways to provide the GPG key material:

### Compile the HashiCorp GPG Key into the Application

This is the default way in which the GPG key is provided.

`hcdl` provides the `embed_gpg_key` feature to compile the GPG key directly
into the application. It will use the GPG key provided at `gpg/hashicorp.asc`
in the source repository. You are encouraged to check the validity of this GPG
key using the steps above before using this feature.

Once you are happy that the GPG key is valid, you can compile `hcdl` as follows
to enable the GPG key embedding:

```shell
$ cargo build --release
```

### Provide `hashicorp.asc` via the Filesystem

To enable this, place a file named `hashicorp.asc` containing the HashiCorp GPG
key from https://www.hashicorp.com/security into an appropriate directory for
your operating system, according to the following table:

| Operating System | Path                                          |
|------------------|-----------------------------------------------|
| macOS            | `~/Library/Application Support/hcdl`          |
| Windows          | `%APPDATA%/hcdl`                              |
| Other            | `$XDG_DATA_DIR/hcdl` or `~/.local/share/hcdl` |

To build `hcdl` with without the embedded GPG key, use the following command:

```shell
$ cargo build --release --no-default-features
```

### Verifying the GPG Key

The key you place here should match the key ID and fingerprint shown on the
security page, and they can be checked as follows:

```shell
$ gpg \
    --dry-run \
    --import \
    --import-options import-show \
    hashicorp.asc
```

Which should result in the following output:

```
pub   rsa2048 2014-02-26 [SC]
      91A6E7F85D05C65630BEF18951852D87348FFC4C
uid                      HashiCorp Security <security@hashicorp.com>
sub   rsa2048 2014-02-26 [E] [expires: 2024-03-25]

gpg: Total number processed: 1
```

At the time of writing, the GPG key stored within this repository is GPG key
ID `51852D87348FFC4C` with fingerprint
`91A6 E7F8 5D05 C656 30BE F189 5185 2D87 348F FC4C`, which should match the GPG
key published by HashiCorp.

If a GPG key isn't present and you still wish to use the tool, you will be
required to explicitly disable the signature verification with the
`--no-verify-signature` flag. Running `hcdl` by disabling the GPG signature
verification is NOT recommended.

## Examples

The following examples were gathered on an x86\_64 FreeBSD machine.

### Checking for the latest Terraform version

```shell
$ hcdl --check terraform
Latest version: terraform v0.12.26 from Wed, 27 May 2020 17:17:12 +0000
```

### Retaining Downloaded Zip File after Installation

```
$ hcdl --keep terraform
Latest version: terraform v0.12.28 from Thu, 25 Jun 2020 16:21:37 +0000
Downloading and verifying signature of terraform_0.12.28_SHA256SUMS...
Verified against terraform_0.12.28_SHA256SUMS.sig.
Downloading terraform_0.12.28_freebsd_amd64.zip...
  [00:00:07] [########################################] 27.07MB/27.07MB (0s) done.
SHA256 of terraform_0.12.28_freebsd_amd64.zip OK.
Unzipping contents of 'terraform_0.12.28_freebsd_amd64.zip' to '/home/user/.local/bin'
-> Extracting 'terraform' to '/home/user/.local/bin'...
Installation successful.
Keeping zipfile terraform_0.12.28_freebsd_amd64.zip in current directory.
```

### Download a Product for a Different OS

```
$ hcdl --os linux terraform
Latest version: terraform v0.12.28 from Thu, 25 Jun 2020 16:21:37 +0000
Downloading and verifying signature of terraform_0.12.28_SHA256SUMS...
Verified against terraform_0.12.28_SHA256SUMS.sig.
Downloading terraform_0.12.28_linux_amd64.zip...
  [00:00:04] [########################################] 27.11MB/27.11MB (0s) done.
SHA256 of terraform_0.12.28_linux_amd64.zip OK.
Product downloaded for different OS, freebsd != linux
Skipping install and keeping zipfile 'terraform_0.12.28_linux_amd64.zip' in current directory.
```

### Download a Specific Version of a Product

```
$ hcdl --build 0.12.25 terraform
Downloading and verifying signature of terraform_0.12.25_SHA256SUMS...
Verified against terraform_0.12.25_SHA256SUMS.sig.
Downloading terraform_0.12.25_freebsd_amd64.zip...
  [00:00:03] [########################################] 15.97MB/15.97MB (0s) done.
SHA256 of terraform_0.12.25_freebsd_amd64.zip OK.
Unzipping contents of 'terraform_0.12.25_freebsd_amd64.zip' to '/home/user/.local/bin'
-> Extracting 'terraform' to '/home/user/.local/bin'...
Installation successful.
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE] or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT] or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

<!-- links -->
[crates.io]: https://crates.io/crates/hcdl
[FreeBSD]: https://www.freebsd.org/
[HashiCorp]: https://www.hashicorp.com/
[LICENSE-APACHE]: LICENSE-APACHE
[LICENSE-MIT]: LICENSE-MIT
[Rust]: https://www.rust-lang.org/
[Terraform]: https://www.terraform.io/
