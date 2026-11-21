//! What a model is to an envelope: the id its kept mint gave it, and the properties its struct
//! declares under theirs.

use std::fmt;

use ankurah::EntityId;

use super::{EncodeError, Properties, PropertyWriter, ReadError};

#[cfg(test)]
mod tests;

/// The id a kept mint gives a model or a property: the 32 bytes of the entity id of its
/// definition. An envelope keys each component, and each of its properties, by it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MintedId([u8; 32]);

impl MintedId {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        MintedId(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Written as `#[model(id = ...)]` and `#[property(id = ...)]` write it, so that an id an error
/// names can be found in the source.
impl fmt::Display for MintedId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&EntityId::from_bytes(self.0).to_base64())
    }
}

impl fmt::Debug for MintedId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// A model that travels in an envelope as a component: an Ankurah model whose struct declares the
/// minted ids of the model and of each of its properties, as the mint writes them. Every model of
/// the Koinon Commons is one, so that any of them can be sent as well as stored, and a model of a
/// crate built on the Commons is one by the same derive.
///
/// `#[derive(Minted)]` implements it beside `#[derive(Model)]`, from the same attributes. It
/// carries each property in the form of its value type, converted through the type's `Property`
/// impl as a stored record's property is, so that a value the type refuses in a record it refuses
/// in an envelope. An optional property that holds nothing is left out, and reads back as `None`.
/// A field marked `#[model(ephemeral)]` is not sent, and reads as its default. A property whose
/// value type has no form in an envelope fails where the field is declared:
///
/// ```compile_fail,E0080
/// use ankurah::{property::PropertyError, Model, Property, Value};
/// use koinon_commons::codec::Minted;
///
/// /// An amount in a value type of a later Ankurah, which no envelope carries yet.
/// #[derive(Debug, Clone)]
/// pub struct Amount(String);
///
/// impl Property for Amount {
///     const VALUE_TYPE: &'static str = "decimal";
///     fn into_value(&self) -> Result<Option<Value>, PropertyError> {
///         Ok(Some(Value::String(self.0.clone())))
///     }
///     fn from_value(_: Option<Value>) -> Result<Self, PropertyError> {
///         Err(PropertyError::Missing)
///     }
/// }
///
/// #[derive(Model, Minted, Debug, Clone)]
/// #[model(id = "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE")]
/// #[model(no_ffi)]
/// pub struct Price {
///     #[active_type(LWW)]
///     #[property(id = "AgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgI")]
///     pub amount: Amount,
/// }
/// # fn main() {}
/// ```
pub trait Minted: ankurah::Model {
    /// The model's minted id, as `#[model(id = ...)]` declares it.
    const ID: MintedId;

    /// Writes each property that holds a value under its minted id.
    fn write_properties(&self, properties: &mut PropertyWriter) -> Result<(), EncodeError>;

    /// Reads the component from its properties, ignoring any its struct does not declare.
    fn read_properties(properties: &Properties<'_>) -> Result<Self, ReadError>;
}
