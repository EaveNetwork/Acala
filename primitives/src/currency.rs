// This file is part of Acala.

// Copyright (C) 2020-2021 Acala Foundation.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

#![allow(clippy::from_over_into)]

use crate::evm::EvmAddress;
use bstringify::bstringify;
use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use sp_std::{
	convert::{Into, TryFrom, TryInto},
	prelude::*,
};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

macro_rules! create_currency_id {
    ($(#[$meta:meta])*
	$vis:vis enum TokenSymbol {
        $($(#[$vmeta:meta])* $symbol:ident($name:expr, $deci:literal) = $val:literal,)*
    }) => {
        $(#[$meta])*
        $vis enum TokenSymbol {
            $($(#[$vmeta])* $symbol = $val,)*
        }

        impl TryFrom<u8> for TokenSymbol {
            type Error = ();

            fn try_from(v: u8) -> Result<Self, Self::Error> {
                match v {
                    $($val => Ok(TokenSymbol::$symbol),)*
                    _ => Err(()),
                }
            }
        }

		impl TryFrom<Vec<u8>> for CurrencyId {
			type Error = ();
			fn try_from(v: Vec<u8>) -> Result<CurrencyId, ()> {
				match v.as_slice() {
					$(bstringify!($symbol) => Ok(CurrencyId::Token(TokenSymbol::$symbol)),)*
					_ => Err(()),
				}
			}
		}

		impl GetDecimals for CurrencyId {
			fn decimals(&self) -> Option<u8> {
				match self {
					$(CurrencyId::Token(TokenSymbol::$symbol) => Some($deci),)*
					_ => None,
				}
			}
		}

		$(pub const $symbol: CurrencyId = CurrencyId::Token(TokenSymbol::$symbol);)*

		impl TokenSymbol {
			pub fn get_info() -> Vec<(&'static str, u32)> {
				vec![
					$((stringify!($symbol), $deci),)*
				]
			}
		}

		#[test]
		#[ignore]
		fn generate_token_resources() {
			#[allow(non_snake_case)]
			#[derive(Serialize, Deserialize)]
			struct Token {
				name: String,
				symbol: String,
				decimals: u8,
				currencyId: u8,
			}

			let tokens = vec![
				$(
					Token {
						name: $name.to_string(),
						symbol: stringify!($symbol).to_string(),
						decimals: $deci,
						currencyId: $val,
					},
				)*
			];
			frame_support::assert_ok!(std::fs::write("../predeploy-contracts/resources/tokens.json", serde_json::to_string_pretty(&tokens).unwrap()));
		}
    }
}

create_currency_id! {
	// Represent a Token symbol with 8 bit
	// Bit 8 : 0 for Pokladot Ecosystem, 1 for Kusama Ecosystem
	// Bit 7 : Reserved
	// Bit 6 - 1 : The token ID
	#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	#[repr(u8)]
	pub enum TokenSymbol {
		// Polkadot Ecosystem
		ACA("Acala", 12) = 0,
		AUSD("Acala Dollar", 12) = 1,
		DOT("Polkadot", 10) = 2,
		LDOT("Liquid DOT", 10) = 3,
		XBTC("ChainX BTC", 8) = 4,
		RENBTC("Ren Protocol BTC", 8) = 5,
		POLKABTC("PolkaBTC", 8) = 6,
		PLM("Plasm", 18) = 7,
		PHA("Phala", 12) = 8,
		HDT("HydraDX", 12) = 9,
		BCG("Bit.Country", 18) = 11,

		// Kusama Ecosystem
		KAR("Karura", 12) = 128,
		KUSD("Karura Dollar", 12) = 129,
		KSM("Kusama", 12) = 130,
		LKSM("Liquid KSM", 12) = 131,
		// Reserve for XBTC = 132
		// Reserve for RENBTC = 133
		// Reserve for POLKABTC = 134
		SDN("Shiden", 18) = 135,
		// Reserve for PHA = 136
		// Reserve for HDT = 137
		KILT("Kilt", 15) = 138,
		// Reserve for BCG = 139
	}
}

pub trait GetDecimals {
	fn decimals(&self) -> Option<u8>;
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum DexShare {
	Token(TokenSymbol),
	Erc20(EvmAddress),
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CurrencyId {
	Token(TokenSymbol),
	DexShare(DexShare, DexShare),
	Erc20(EvmAddress),
}

impl CurrencyId {
	pub fn is_token_currency_id(&self) -> bool {
		matches!(self, CurrencyId::Token(_))
	}

	pub fn is_dex_share_currency_id(&self) -> bool {
		matches!(self, CurrencyId::DexShare(_, _))
	}

	pub fn is_erc20_currency_id(&self) -> bool {
		matches!(self, CurrencyId::Erc20(_))
	}

	pub fn split_dex_share_currency_id(&self) -> Option<(Self, Self)> {
		match self {
			CurrencyId::DexShare(token_symbol_0, token_symbol_1) => {
				let symbol_0 = match token_symbol_0 {
					DexShare::Token(token) => CurrencyId::Token(*token),
					DexShare::Erc20(address) => CurrencyId::Erc20(*address),
				};
				let symbol_1 = match token_symbol_1 {
					DexShare::Token(token) => CurrencyId::Token(*token),
					DexShare::Erc20(address) => CurrencyId::Erc20(*address),
				};
				Some((symbol_0, symbol_1))
			}
			_ => None,
		}
	}

	pub fn join_dex_share_currency_id(currency_id_0: Self, currency_id_1: Self) -> Option<Self> {
		let token_symbol_0 = match currency_id_0 {
			CurrencyId::Token(symbol) => DexShare::Token(symbol),
			CurrencyId::Erc20(address) => DexShare::Erc20(address),
			_ => return None,
		};
		let token_symbol_1 = match currency_id_1 {
			CurrencyId::Token(symbol) => DexShare::Token(symbol),
			CurrencyId::Erc20(address) => DexShare::Erc20(address),
			_ => return None,
		};
		Some(CurrencyId::DexShare(token_symbol_0, token_symbol_1))
	}
}

/// Note the pre-deployed Erc20 contracts depend on `CurrencyId` implementation,
/// and need to be updated if any change.
impl TryFrom<[u8; 32]> for CurrencyId {
	type Error = ();

	fn try_from(v: [u8; 32]) -> Result<Self, Self::Error> {
		// token/dex flag(1byte) | evm address(20byte)
		// token/dex flag(1byte) | dex left(4byte) | dex right(4byte)
		// v[11] = token/dex flag(1byte)
		// v[12..16] = dex left(4byte)
		// v[16..20] = dex right(4byte)
		// v[12..32] = evm address(20byte)

		if !v.starts_with(&[0u8; 11][..]) {
			return Err(());
		}

		// token
		if v[11] == 0 && v[12..15] == [0u8; 3] && v[16..32] == [0u8; 16] {
			return v[15].try_into().map(CurrencyId::Token);
		}

		// erc20
		if v[11] == 0 {
			return Ok(CurrencyId::Erc20(EvmAddress::from_slice(&v[12..32])));
		}

		Err(())
	}
}

impl TryFrom<CurrencyId> for u32 {
	type Error = ();

	fn try_from(val: CurrencyId) -> Result<Self, Self::Error> {
		let mut bytes = [0u8; 4];
		match val {
			CurrencyId::Token(token) => {
				bytes[3] = token as u8;
			}
			CurrencyId::Erc20(address) => {
				let is_zero = |&&d: &&u8| -> bool { d == 0 };
				let leading_zeros = address.as_bytes().iter().take_while(is_zero).count();
				let index = if leading_zeros > 16 { 16 } else { leading_zeros };
				bytes[..].copy_from_slice(&address[index..index + 4][..]);
			}
			_ => {
				return Err(());
			}
		}
		Ok(u32::from_be_bytes(bytes))
	}
}

impl Into<CurrencyId> for DexShare {
	fn into(self) -> CurrencyId {
		match self {
			DexShare::Token(token) => CurrencyId::Token(token),
			DexShare::Erc20(address) => CurrencyId::Erc20(address),
		}
	}
}
