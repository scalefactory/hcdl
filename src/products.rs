// products: List of products that hcdl can be used with
#![forbid(unsafe_code)]
#![forbid(missing_docs)]

/// A list of [HashiCorp](https://www.hashicorp.com/) products that this crate
/// can download.
pub const PRODUCTS_LIST: &[&str] = &[
    "consul",
    "nomad",
    "packer",
    "terraform",
    "vagrant",
    "vault",
];
