//! Enums stored and carried as the name of a variant, in snake case, so that a query reads
//! `state = 'accepted'`, and a name a later build adds does not read as one this build knows.

/// Declares `$enum`, a unit enum whose every variant is written `$variant = "name",`, with every
/// variant in `ALL`, in the order declared; each variant's name, and the variant a name names; its
/// name as its `Display`; and its `Property`, which stores a variant as its name and reads no other
/// text: a name this build does not know is a value the enum refuses. The enum derives `Debug`,
/// `Clone`, `Copy`, `PartialEq`, `Eq` and `Hash`.
macro_rules! enum_stored_by_name {
    (
        $(#[$doc:meta])*
        pub enum $enum:ident {
            $($(#[$variant_doc:meta])* $variant:ident = $name:literal,)+
        }
    ) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $enum {
            $($(#[$variant_doc])* $variant,)+
        }

        impl $enum {
            /// Every variant, in the order declared.
            pub const ALL: &'static [$enum] = &[$($enum::$variant),+];

            /// The variant's name, as it is stored.
            pub fn name(self) -> &'static str {
                match self {
                    $($enum::$variant => $name,)+
                }
            }

            /// The variant whose name is `name`, or None for a name this build does not know.
            pub fn from_name(name: &str) -> Option<Self> {
                Self::ALL.iter().copied().find(|variant| variant.name() == name)
            }
        }

        impl ::std::fmt::Display for $enum {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(self.name())
            }
        }

        impl ::ankurah::Property for $enum {
            const VALUE_TYPE: &'static str = "string";

            fn into_value(&self) -> Result<Option<::ankurah::value::Value>, ::ankurah::property::PropertyError> {
                Ok(Some(::ankurah::value::Value::String(self.name().to_owned())))
            }

            fn from_value(value: Option<::ankurah::value::Value>) -> Result<Self, ::ankurah::property::PropertyError> {
                use ::ankurah::{property::PropertyError, value::Value};
                match value {
                    Some(Value::String(name)) => {
                        $enum::from_name(&name).ok_or(PropertyError::InvalidValue { value: name, ty: stringify!($enum).to_owned() })
                    }
                    Some(other) => Err(PropertyError::InvalidVariant { given: other, ty: stringify!($enum).to_owned() }),
                    None => Err(PropertyError::Missing),
                }
            }
        }
    };
}

pub(crate) use enum_stored_by_name;
