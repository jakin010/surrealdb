//! Stores the key prefix for all keys under a namespace access method
use crate::key::category::Categorise;
use crate::key::category::Category;
use crate::kvs::impl_key;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Access<'a> {
	__: u8,
	_a: u8,
	pub ns: &'a str,
	_b: u8,
	pub ac: &'a str,
}
impl_key!(Access<'a>);

pub fn new<'a>(ns: &'a str, ac: &'a str) -> Access<'a> {
	Access::new(ns, ac)
}

impl Categorise for Access<'_> {
	fn categorise(&self) -> Category {
		Category::NamespaceAccessRoot
	}
}

impl<'a> Access<'a> {
	pub fn new(ns: &'a str, ac: &'a str) -> Self {
		Self {
			__: b'/',
			_a: b'*',
			ns,
			_b: b'&',
			ac,
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::kvs::{KeyDecode, KeyEncode};
	#[test]
	fn key() {
		use super::*;
		#[rustfmt::skip]
		let val = Access::new(
			"testns",
			"testac",
		);
		let enc = Access::encode(&val).unwrap();
		assert_eq!(enc, b"/*testns\0&testac\0");

		let dec = Access::decode(&enc).unwrap();
		assert_eq!(val, dec);
	}
}
