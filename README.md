# `hcdl`: HashiCorp Downloader

`hcdl` is a tool for easily downloading and (optionally) installing
[HashiCorp] products.

## Usage

`hcdl` usage is very simple, for example, if we want to download and install
the latest version of [Terraform], we can run the following:

```shell
$ hcdl --install terraform
```

You'll see output like the following:

```shell
Downloading and verifying signature of terraform_0.12.26_SHA256SUMS...
  Verified against terraform_0.12.26_SHA256SUMS.sig.
Downloading terraform_0.12.26_freebsd_amd64.zip...
  [00:00:02] [########################################] 16.06MB/16.06MB (0s) done.
SHA256 of terraform_0.12.26_freebsd_amd64.zip OK.
Unzipping 'terraform' from 'terraform_0.12.26_freebsd_amd64.zip' to '/home/user/.local/bin'...
  Installation successful.
```

`hcdl` has performed the following steps:

  - Loaded the HashiCorp GPG key
  - Found the latest version of Terraform
  - Downloaded the SHASUM256 GPG signature file
  - Downloaded the SHASUM256 file
  - Verified the SHASUM256 file against the signature
  - Downloaded the latest version of Terraform
  - Verified that the SHA256 of the downloaded file matches the SHASUM256 file
  - Installed the `terraform` binary to `~/.local/bin`

By default, `hcdl` will download products for the operating system and
architecture that it was compiled for (above we were running `hcdl` on an
x86\_64 [FreeBSD] machine), however, you can download any product for any OS
and architecture you like by specifying the `--os` and `--arch` options.

## HashiCorp GPG Key

Due to the GPG signature checking, `hcdl` needs to know the HashiCorp GPG key.
To enable this, place the HashiCorp GPG key from
https://www.hashicorp.com/security into the `~/.local/share/hcdl` directory as
a file named `hashicorp.asc`.

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
`--no-verify-signature` flag.

<!-- links -->
[FreeBSD]: https://www.freebsd.org/
[HashiCorp]: https://www.hashicorp.com/
[Terraform]: https://www.terraform.io/
