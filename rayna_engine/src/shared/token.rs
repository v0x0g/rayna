#[rustfmt::skip] // rustfmt issue #5974
macro_rules! generate_component_token {
    ($token_type:ident for $inst_type:ty) => {
        #[doc = ::const_format::formatcp!(
            "\
            An identifier used as a reference for a {inst_type}, stored inside a scene\
            \
            See the scene documentation for details on tokens\
            ",
            inst_type = stringify!($inst_type)
        )]
        #[derive(
            Debug,
            Copy, Clone,
            Ord, PartialOrd, Eq, PartialEq, Hash
            valuable::Valuable,
        )]
        pub struct $token_type(pub $crate::core::types::IdToken);

        impl std::fmt::Display for $token_type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{id:>0width$X}",
                    id = self.0,
                    width = $crate::core::types::IdToken::BITS as usize / 4
                )
            }
        }
    };
}

// Export
pub(crate) use generate_component_token;
