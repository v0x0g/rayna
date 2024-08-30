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

/// Generates a token type, with optional generics on the token type
///
/// # Generics
///
/// If adding generics to the token, they should be added as shown in the examples. The odd syntax
/// Is a requirement for the way macros are parsed (since `<` and `>` are not matching pairs like braces are).
///
/// # Example
/// 
/// ```
/// // Generates a token type `NoiseToken`, which has the generic parameters `<const N : usize>`
/// // Which is used as `NoiseToken<N>`
/// rayna_engine::core::token::generate_component_token!(NoiseToken < {const N: usize} as {N} > for NoiseInstance);
/// // Generates a `MaterialToken`, without generics
/// rayna_engine::core::token::generate_component_token!(MaterialToken for MaterialInstance);
/// ```
#[rustfmt::skip] // rustfmt issue #5974
#[allow(rustdoc::private_doc_tests)] // Publicly exported below
macro_rules! generate_component_token {
    ($token_type:ident $(< {$($token_generic_tt:tt)+} as {$($token_generic_name:ident),+} >)? for $inst_type:ty) => {
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
            derive_more::From,
            derive_more::Display,
        )]
        #[display("{id:>0width$X}", id = _0, width = 16)]
        pub struct $token_type $(<$($token_generic_tt)+>)? (
            pub $crate::core::token::IdToken
        );
    };
}

// Export
pub(crate) use generate_component_token;
