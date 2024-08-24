/// Numeric identifier used to uniquely mark components
#[derive(
    Copy,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Hash,
    Ord,
    PartialOrd,
    valuable::Valuable,
    derive_more::From,
    derive_more::Display,
    derive_more::UpperHex,
    derive_more::LowerHex,
    derive_more::Binary,
    derive_more::Octal,
)]
pub struct IdToken(u64);

#[rustfmt::skip] // rustfmt issue #5974
macro_rules! generate_component_token {
    ($token_type:ident for $inst_type:ty) => {
        #[doc = concat!(
            "An identifier used as a reference for a ", stringify!(inst_type), " stored inside a scene \
            \
            See the scene documentation for details on tokens",
        )]
        #[derive(
            Debug,
            Copy, Clone,
            Ord, PartialOrd, Eq, PartialEq, Hash,
            valuable::Valuable,
        )]
        pub struct $token_type(pub $crate::core::token::IdToken);

        impl std::fmt::Display for $token_type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{id:>0width$X}",
                    id = self.0,
                    width = u64::BITS as usize / 4
                )
            }
        }

        impl core::convert::From<$crate::core::token::IdToken> for $token_type {
            fn from(value: $crate::core::token::IdToken) -> Self {
                Self(value)
            }
        }
    };
}

// Export
pub(crate) use generate_component_token;
