use super::{U128, U256, U512, U64};
use serde::{Deserialize, Serialize, Serializer};
use std::{
    convert::{TryFrom, TryInto},
    fmt,
    time::Duration,
};
use strum::{AsRefStr, EnumCount, EnumIter, EnumString, EnumVariantNames};

// compatibility re-export
#[doc(hidden)]
pub use num_enum::{TryFromPrimitive, TryFromPrimitiveError};
#[doc(hidden)]
pub type ParseChainError = TryFromPrimitiveError<Chain>;

// When adding a new chain:
//   1. add new variant to the Chain enum;
//   2. add extra information in the last `impl` block (explorer URLs, block time) when applicable;
//   3. (optional) add aliases: `#[strum(serialize = "main", serialize = "alias", ...)]`;
//      "main" must be present and will be used in `Display`, `Serialize` and `FromStr`,
//      while the aliases will be added only to `FromStr`.

/// An Ethereum EIP-155 chain.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    AsRefStr,         // also for fmt::Display and serde::Serialize
    EnumVariantNames, // Self::VARIANTS
    EnumString,       // FromStr, TryFrom<&str>
    EnumIter,
    EnumCount,
    TryFromPrimitive, // TryFrom<u64>
    Deserialize,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "kebab-case")]
#[repr(u64)]
pub enum Chain {
    Mainnet = 1,
    Morden = 2,
    Ropsten = 3,
    Rinkeby = 4,
    Goerli = 5,
    Kovan = 42,
    Sepolia = 11155111,

    Optimism = 10,
    OptimismKovan = 69,
    OptimismGoerli = 420,

    Arbitrum = 42161,
    ArbitrumTestnet = 421611,
    ArbitrumGoerli = 421613,
    ArbitrumNova = 42170,

    Cronos = 25,
    CronosTestnet = 338,

    Rsk = 30,

    #[strum(serialize = "bsc")]
    BinanceSmartChain = 56,
    #[strum(serialize = "bsc-testnet")]
    BinanceSmartChainTestnet = 97,

    Poa = 99,
    Sokol = 77,

    #[strum(serialize = "gnosis", serialize = "xdai", serialize = "gnosis-chain")]
    XDai = 100,

    Polygon = 137,
    #[strum(serialize = "mumbai", serialize = "polygon-mumbai")]
    PolygonMumbai = 80001,

    Fantom = 250,
    FantomTestnet = 4002,

    Moonbeam = 1284,
    MoonbeamDev = 1281,

    Moonriver = 1285,

    Moonbase = 1287,

    #[strum(serialize = "dev")]
    Dev = 1337,
    #[strum(serialize = "anvil-hardhat", serialize = "anvil", serialize = "hardhat")]
    AnvilHardhat = 31337,

    Evmos = 9001,
    EvmosTestnet = 9000,

    Chiado = 10200,

    Oasis = 26863,

    Emerald = 42262,
    EmeraldTestnet = 42261,

    Avalanche = 43114,
    #[strum(serialize = "fuji", serialize = "avalanche-fuji")]
    AvalancheFuji = 43113,

    Celo = 42220,
    CeloAlfajores = 44787,
    CeloBaklava = 62320,

    Aurora = 1313161554,
    AuroraTestnet = 1313161555,
}

// === impl Chain ===

// This must be implemented manually so we avoid a conflict with `TryFromPrimitive` where it treats
// the `#[default]` attribute as its own `#[num_enum(default)]`
impl Default for Chain {
    fn default() -> Self {
        Self::Mainnet
    }
}

macro_rules! impl_into_numeric {
    ($($ty:ty)+) => {$(
        impl From<Chain> for $ty {
            fn from(chain: Chain) -> Self {
                u64::from(chain).into()
            }
        }
    )+};
}

macro_rules! impl_try_from_numeric {
    ($($native:ty)+ ; $($primitive:ty)*) => {
        $(
            impl TryFrom<$native> for Chain {
                type Error = ParseChainError;

                fn try_from(value: $native) -> Result<Self, Self::Error> {
                    (value as u64).try_into()
                }
            }
        )+

        $(
            impl TryFrom<$primitive> for Chain {
                type Error = ParseChainError;

                fn try_from(value: $primitive) -> Result<Self, Self::Error> {
                    if value.bits() > 64 {
                        // `TryFromPrimitiveError` only has a `number` field which has the same type
                        // as the `#[repr(_)]` attribute on the enum.
                        return Err(ParseChainError { number: value.low_u64() })
                    }
                    value.low_u64().try_into()
                }
            }
        )*
    };
}

impl From<Chain> for u64 {
    fn from(chain: Chain) -> Self {
        chain as u64
    }
}

impl_into_numeric!(u128 U64 U128 U256 U512);

impl TryFrom<U64> for Chain {
    type Error = ParseChainError;

    fn try_from(value: U64) -> Result<Self, Self::Error> {
        value.low_u64().try_into()
    }
}

impl_try_from_numeric!(u8 u16 u32 usize; U128 U256 U512);

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(self.as_ref())
    }
}

impl Serialize for Chain {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_str(self.as_ref())
    }
}

// NB: all utility functions *should* be explicitly exhaustive (not use `_` matcher) so we don't
//     forget to update them when adding a new `Chain` variant.
impl Chain {
    /// Returns the chain's average blocktime, if applicable.
    ///
    /// It can be beneficial to know the average blocktime to adjust the polling of an HTTP provider
    /// for example.
    ///
    /// **Note:** this is not an accurate average, but is rather a sensible default derived from
    /// blocktime charts such as [Etherscan's](https://etherscan.com/chart/blocktime)
    /// or [Polygonscan's](https://polygonscan.com/chart/blocktime).
    pub const fn average_blocktime_hint(&self) -> Option<Duration> {
        use Chain::*;

        let ms = match self {
            Arbitrum | ArbitrumTestnet | ArbitrumGoerli | ArbitrumNova => 1_300,
            Mainnet | Optimism => 13_000,
            Polygon | PolygonMumbai => 2_100,
            Moonbeam | Moonriver => 12_500,
            BinanceSmartChain | BinanceSmartChainTestnet => 3_000,
            Avalanche | AvalancheFuji => 2_000,
            Fantom | FantomTestnet => 1_200,
            Cronos | CronosTestnet => 5_700,
            Evmos | EvmosTestnet => 1_900,
            Aurora | AuroraTestnet => 1_100,
            Oasis => 5_500,
            Emerald => 6_000,
            Dev | AnvilHardhat => 200,
            Celo | CeloAlfajores | CeloBaklava => 5_000,
            // Explictly handle all network to make it easier not to forget this match when new
            // networks are added.
            Morden | Ropsten | Rinkeby | Goerli | Kovan | XDai | Chiado | Sepolia | Moonbase |
            MoonbeamDev | OptimismGoerli | OptimismKovan | Poa | Sokol | Rsk | EmeraldTestnet => {
                return None
            }
        };

        Some(Duration::from_millis(ms))
    }

    /// Returns the chain's blockchain explorer and its API (Etherscan and Etherscan-like) URLs.
    ///
    /// Returns `(API URL, BASE_URL)`, like `("https://api(-chain).etherscan.io/api", "https://etherscan.io")`
    pub const fn etherscan_urls(&self) -> Option<(&'static str, &'static str)> {
        use Chain::*;

        let urls = match self {
            Mainnet => ("https://api.etherscan.io/api", "https://etherscan.io"),
            Ropsten => ("https://api-ropsten.etherscan.io/api", "https://ropsten.etherscan.io"),
            Kovan => ("https://api-kovan.etherscan.io/api", "https://kovan.etherscan.io"),
            Rinkeby => ("https://api-rinkeby.etherscan.io/api", "https://rinkeby.etherscan.io"),
            Goerli => ("https://api-goerli.etherscan.io/api", "https://goerli.etherscan.io"),
            Sepolia => ("https://api-sepolia.etherscan.io/api", "https://sepolia.etherscan.io"),
            Polygon => ("https://api.polygonscan.com/api", "https://polygonscan.com"),
            PolygonMumbai => {
                ("https://api-testnet.polygonscan.com/api", "https://mumbai.polygonscan.com")
            }
            Avalanche => ("https://api.snowtrace.io/api", "https://snowtrace.io"),
            AvalancheFuji => {
                ("https://api-testnet.snowtrace.io/api", "https://testnet.snowtrace.io")
            }
            Optimism => {
                ("https://api-optimistic.etherscan.io/api", "https://optimistic.etherscan.io")
            }
            OptimismGoerli => (
                "https://api-goerli-optimistic.etherscan.io/api",
                "https://goerli-optimism.etherscan.io",
            ),
            OptimismKovan => (
                "https://api-kovan-optimistic.etherscan.io/api",
                "https://kovan-optimistic.etherscan.io",
            ),
            Fantom => ("https://api.ftmscan.com/api", "https://ftmscan.com"),
            FantomTestnet => ("https://api-testnet.ftmscan.com/api", "https://testnet.ftmscan.com"),
            BinanceSmartChain => ("https://api.bscscan.com/api", "https://bscscan.com"),
            BinanceSmartChainTestnet => {
                ("https://api-testnet.bscscan.com/api", "https://testnet.bscscan.com")
            }
            Arbitrum => ("https://api.arbiscan.io/api", "https://arbiscan.io"),
            ArbitrumTestnet => {
                ("https://api-testnet.arbiscan.io/api", "https://testnet.arbiscan.io")
            }
            ArbitrumGoerli => (
                "https://goerli-rollup-explorer.arbitrum.io/api",
                "https://goerli-rollup-explorer.arbitrum.io",
            ),
            ArbitrumNova => ("https://api-nova.arbiscan.io/api", "https://nova.arbiscan.io/"),
            Cronos => ("https://api.cronoscan.com/api", "https://cronoscan.com"),
            CronosTestnet => {
                ("https://api-testnet.cronoscan.com/api", "https://testnet.cronoscan.com")
            }
            Moonbeam => ("https://api-moonbeam.moonscan.io/api", "https://moonbeam.moonscan.io/"),
            Moonbase => ("https://api-moonbase.moonscan.io/api", "https://moonbase.moonscan.io/"),
            Moonriver => ("https://api-moonriver.moonscan.io/api", "https://moonriver.moonscan.io"),
            // blockscout API is etherscan compatible
            XDai => {
                ("https://blockscout.com/xdai/mainnet/api", "https://blockscout.com/xdai/mainnet")
            }
            Chiado => {
                ("https://blockscout.chiadochain.net/api", "https://blockscout.chiadochain.net")
            }
            Sokol => ("https://blockscout.com/poa/sokol/api", "https://blockscout.com/poa/sokol"),
            Poa => ("https://blockscout.com/poa/core/api", "https://blockscout.com/poa/core"),
            Rsk => ("https://blockscout.com/rsk/mainnet/api", "https://blockscout.com/rsk/mainnet"),
            Oasis => ("https://scan.oasischain.io/api", "https://scan.oasischain.io/"),
            Emerald => {
                ("https://explorer.emerald.oasis.dev/api", "https://explorer.emerald.oasis.dev/")
            }
            EmeraldTestnet => (
                "https://testnet.explorer.emerald.oasis.dev/api",
                "https://testnet.explorer.emerald.oasis.dev/",
            ),
            Aurora => ("https://api.aurorascan.dev/api", "https://aurorascan.dev"),
            AuroraTestnet => {
                ("https://testnet.aurorascan.dev/api", "https://testnet.aurorascan.dev")
            }
            Evmos => ("https://evm.evmos.org/api", "https://evm.evmos.org/"),
            EvmosTestnet => ("https://evm.evmos.dev/api", "https://evm.evmos.dev/"),
            Celo => ("https://explorer.celo.org/mainnet", "https://explorer.celo.org/mainnet/api"),
            CeloAlfajores => {
                ("https://explorer.celo.org/alfajores", "https://explorer.celo.org/alfajores/api")
            }
            CeloBaklava => {
                ("https://explorer.celo.org/baklava", "https://explorer.celo.org/baklava/api")
            }
            AnvilHardhat | Dev | Morden | MoonbeamDev => {
                // this is explicitly exhaustive so we don't forget to add new urls when adding a
                // new chain
                return None
            }
        };

        Some(urls)
    }

    /// Returns whether the chain implements EIP-1559 (with the type 2 EIP-2718 transaction type).
    pub const fn is_legacy(&self) -> bool {
        use Chain::*;

        match self {
            // Known legacy chains / non EIP-1559 compliant
            Optimism |
            OptimismGoerli |
            OptimismKovan |
            Fantom |
            FantomTestnet |
            BinanceSmartChain |
            BinanceSmartChainTestnet |
            Arbitrum |
            ArbitrumTestnet |
            ArbitrumGoerli |
            ArbitrumNova |
            Rsk |
            Oasis |
            Emerald |
            EmeraldTestnet |
            Celo |
            CeloAlfajores |
            CeloBaklava => true,

            // Known EIP-1559 chains
            Mainnet | Goerli | Sepolia | Polygon | PolygonMumbai | Avalanche | AvalancheFuji => {
                false
            }

            // Unknown / not applicable, default to false for backwards compatibility
            Dev | AnvilHardhat | Morden | Ropsten | Rinkeby | Cronos | CronosTestnet | Kovan |
            Sokol | Poa | XDai | Moonbeam | MoonbeamDev | Moonriver | Moonbase | Evmos |
            EvmosTestnet | Chiado | Aurora | AuroraTestnet => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn test_default_chain() {
        assert_eq!(serde_json::to_string(&Chain::default()).unwrap(), "\"mainnet\"");
    }

    #[test]
    fn test_enum_iter() {
        assert_eq!(Chain::COUNT, Chain::iter().size_hint().0);
    }
}
