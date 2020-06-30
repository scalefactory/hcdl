# `hcdl`: HashiCorp Downloader

`hcdl` is a tool for easily downloading and installing [HashiCorp] products.

## Usage

`hcdl` usage is very simple, for example, if we want to download and install
the latest version of [Terraform], we can run the following:

```shell
$ hcdl terraform
```

You'll see output like the following:

```shell
Latest version: terraform v0.12.28 from Thu, 25 Jun 2020 16:21:37 +0000
Downloading and verifying signature of terraform_0.12.28_SHA256SUMS...
Verified against terraform_0.12.28_SHA256SUMS.sig.
Downloading terraform_0.12.28_freebsd_amd64.zip...
  [00:00:04] [########################################] 27.07MB/27.07MB (0s) done.
SHA256 of terraform_0.12.28_freebsd_amd64.zip OK.
Unzipping 'terraform' from 'terraform_0.12.28_freebsd_amd64.zip' to '/home/user/.local/bin'
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
  - Installed the `terraform` binary to `~/.local/bin`

By default, `hcdl` will download products for the operating system and
architecture that it was compiled for (above we were running `hcdl` on an
x86\_64 [FreeBSD] machine), however, you can download any product for any OS
and architecture you like by specifying the `--os` and `--arch` options.

## HashiCorp GPG Key

Due to the GPG signature checking, `hcdl` needs to know the HashiCorp GPG key.
There are two ways to provide the GPG key material:

### Provide `hashicorp.asc` via the Filesystem

To enable this, place a file named `hashicorp.asc` containing the HashiCorp GPG
key from https://www.hashicorp.com/security into an appropriate directory for
your operating system, according to the following table:

| Operating System | Path                                          |
|------------------|-----------------------------------------------|
| macOS            | `~/Library/Application Support/hcdl`          |
| Windows          | `%APPDATA%/hcdl`                              |
| Other            | `$XDG_DATA_DIR/hcdl` or `~/.local/share/hcdl` |

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

### Compile the HashiCorp GPG Key into the Application

`hcdl` provides the `embed_gpg_key` feature to compile the GPG key directly
into the application. It will use the GPG key provided at `gpg/hashicorp.asc`
in the source repository. You are encouraged to check the validity of this GPG
key using the steps above before using this feature.

Once you are happy that the GPG key is valid, you can compile `hcdl` as follows
to enable the GPG key embedding:

```shell
$ cargo build --features=embed_gpg_key --release
```

## Examples

The following examples were gathered on an x86\_64 FreeBSD machine.

### Checking for the latest Terraform version

```shell
$ hcdl --check terraform
Latest version: terraform v0.12.26 from Wed, 27 May 2020 17:17:12 +0000
```

### Retaining Downloaded Zip File after Installation

```shell
$ hcdl --keep terraform
Latest version: terraform v0.12.28 from Thu, 25 Jun 2020 16:21:37 +0000
Downloading and verifying signature of terraform_0.12.28_SHA256SUMS...
Verified against terraform_0.12.28_SHA256SUMS.sig.
Downloading terraform_0.12.28_freebsd_amd64.zip...
  [00:00:07] [########################################] 27.07MB/27.07MB (0s) done.
SHA256 of terraform_0.12.28_freebsd_amd64.zip OK.
Unzipping 'terraform' from 'terraform_0.12.28_freebsd_amd64.zip' to '/home/user/.local/bin'
Installation successful.
Keeping zipfile terraform_0.12.28_freebsd_amd64.zip in current directory.
```

### Download a Product for a Different OS

```shell
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

```shell
$ hcdl --build 0.12.25 terraform
Downloading and verifying signature of terraform_0.12.25_SHA256SUMS...
Verified against terraform_0.12.25_SHA256SUMS.sig.
Downloading terraform_0.12.25_freebsd_amd64.zip...
  [00:00:03] [########################################] 15.97MB/15.97MB (0s) done.
SHA256 of terraform_0.12.25_freebsd_amd64.zip OK.
Unzipping 'terraform' from 'terraform_0.12.25_freebsd_amd64.zip' to '/home/user/.local/bin'
Installation successful.
```

<!-- links -->
[FreeBSD]: https://www.freebsd.org/
[HashiCorp]: https://www.hashicorp.com/
[Terraform]: https://www.terraform.io/
