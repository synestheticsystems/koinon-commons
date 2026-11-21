//! An envelope as a receiver holds it: its bytes exactly as they arrived, which a signature over
//! it is checked against, and an index of its components and records, read into the structs of the
//! components the receiver understands.

use ankurah::{property::PropertyError, EntityId, Property};

use super::{
    structure::{self, ComponentEntry, Index, PropertyEntry},
    value, DecodeError, Limits, Minted, MintedId, ValueProblem,
};

#[cfg(test)]
mod tests;

/// Why components a receiver requires did not read from an envelope.
#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    #[error("component {component} is absent, which this receiver requires")]
    MissingComponent { component: MintedId },
    #[error("component {component} lacks property {property}, which it requires")]
    MissingProperty { component: MintedId, property: MintedId },
    #[error("property {property} of component {component} does not read: {problem}")]
    MalformedValue { component: MintedId, property: MintedId, problem: ValueProblem },
}

/// A received envelope: its bytes, borrowed as they arrived and never encoded again, and where its
/// parts lie in them.
#[derive(Debug)]
pub struct ReceivedEnvelope<'a> {
    bytes: &'a [u8],
    index: Index,
}

impl<'a> ReceivedEnvelope<'a> {
    /// Checks every item of `bytes` within `limits`, and indexes the envelope they hold. No
    /// component is read yet: one the receiver does not know is checked, then ignored.
    ///
    /// `understood` names every component this receiver understands, whatever it reads of them.
    /// An envelope that lists as must-understand a component it does not name is refused, naming
    /// every such component, before anything is read.
    pub fn decode(bytes: &'a [u8], limits: &Limits, understood: &[MintedId]) -> Result<Self, DecodeError> {
        Ok(ReceivedEnvelope { bytes, index: structure::index(bytes, limits, understood)? })
    }

    /// The bytes the envelope was decoded from, as they arrived.
    pub fn bytes(&self) -> &'a [u8] {
        self.bytes
    }

    /// The components the envelope lists as ones a receiver must understand, wherever it holds
    /// them.
    pub fn must_understand(&self) -> &[MintedId] {
        &self.index.must_understand
    }

    /// The envelope's own components, by which a receiver tells what it was sent.
    pub fn components(&self) -> ReceivedComponents<'_> {
        self.held(&self.index.components[self.index.own.clone()])
    }

    /// The records the envelope carries, in the order the sender gave them.
    pub fn records(&self) -> impl ExactSizeIterator<Item = ReceivedRecord<'_>> + '_ {
        self.index.records.iter().map(|record| ReceivedRecord {
            entity: record.entity,
            components: self.held(&self.index.components[record.components.clone()]),
        })
    }

    fn held<'e>(&'e self, components: &'e [ComponentEntry]) -> ReceivedComponents<'e> {
        ReceivedComponents { bytes: self.bytes, components, properties: &self.index.properties }
    }
}

/// One record of a received envelope: an entity's components, and the entity's id unless the
/// record names none.
#[derive(Debug, Clone, Copy)]
pub struct ReceivedRecord<'e> {
    entity: Option<EntityId>,
    components: ReceivedComponents<'e>,
}

impl<'e> ReceivedRecord<'e> {
    pub fn entity(&self) -> Option<EntityId> {
        self.entity
    }

    pub fn components(&self) -> ReceivedComponents<'e> {
        self.components
    }
}

/// Components received together: an envelope's own, or those of one of its records.
#[derive(Debug, Clone, Copy)]
pub struct ReceivedComponents<'e> {
    bytes: &'e [u8],
    /// In the order of their ids.
    components: &'e [ComponentEntry],
    properties: &'e [PropertyEntry],
}

impl<'e> ReceivedComponents<'e> {
    /// Whether component `C` is among them.
    pub fn has<C: Minted>(&self) -> bool {
        self.properties_of(C::ID).is_some()
    }

    /// The ids of every one of them, those the receiver does not know among them.
    pub fn ids(&self) -> impl ExactSizeIterator<Item = MintedId> + 'e {
        self.components.iter().map(|component| component.id)
    }

    /// Reads what `W` names of them: a component required as `C` or optional as `Option<C>`, or a
    /// tuple of these, such as `(Device, Option<Printer>)`.
    pub fn read<W: Wanted>(&self) -> Result<W, ReadError> {
        W::read_from(self)
    }

    fn properties_of(&self, component: MintedId) -> Option<Properties<'e>> {
        let found = self.components.binary_search_by_key(&component, |entry| entry.id).ok()?;
        Some(Properties { component, bytes: self.bytes, entries: &self.properties[self.components[found].properties.clone()] })
    }
}

/// One component's properties in a received envelope, from which `#[derive(Minted)]` reads each
/// property its struct declares.
#[derive(Debug)]
pub struct Properties<'e> {
    component: MintedId,
    bytes: &'e [u8],
    /// In the order of their ids.
    entries: &'e [PropertyEntry],
}

impl Properties<'_> {
    /// Reads `property` as a `T`, converted through its type's `Property` impl as a stored
    /// record's property is: an absent property reads as `None` for an `Option`, and is missing
    /// for any other type.
    pub fn read<T: Property>(&self, property: MintedId) -> Result<T, ReadError> {
        let value_type = const { value::value_type(T::VALUE_TYPE) };
        let component = self.component;
        let malformed = |problem| ReadError::MalformedValue { component, property, problem };
        let value = match self.entries.binary_search_by_key(&property, |entry| entry.id) {
            Ok(found) => Some(value::decode(&self.bytes[self.entries[found].value.clone()], value_type).map_err(malformed)?),
            Err(_) => None,
        };
        let present = value.is_some();
        T::from_value(value).map_err(|error| match error {
            PropertyError::Missing if !present => ReadError::MissingProperty { component, property },
            error => malformed(ValueProblem::Refused(Box::new(error))),
        })
    }
}

/// What one read takes from received components: a component required as `C` or optional as
/// `Option<C>`, or a tuple of these, which may hold tuples in turn.
pub trait Wanted: Sized {
    fn read_from(components: &ReceivedComponents<'_>) -> Result<Self, ReadError>;
}

impl<C: Minted> Wanted for C {
    fn read_from(components: &ReceivedComponents<'_>) -> Result<Self, ReadError> {
        let properties = components.properties_of(C::ID).ok_or(ReadError::MissingComponent { component: C::ID })?;
        C::read_properties(&properties)
    }
}

impl<C: Minted> Wanted for Option<C> {
    fn read_from(components: &ReceivedComponents<'_>) -> Result<Self, ReadError> {
        components.properties_of(C::ID).map(|properties| C::read_properties(&properties)).transpose()
    }
}

macro_rules! wanted_tuple {
    ($($wanted:ident),+) => {
        impl<$($wanted: Wanted),+> Wanted for ($($wanted,)+) {
            fn read_from(components: &ReceivedComponents<'_>) -> Result<Self, ReadError> {
                Ok(($($wanted::read_from(components)?,)+))
            }
        }
    };
}

wanted_tuple!(A, B);
wanted_tuple!(A, B, C);
wanted_tuple!(A, B, C, D);
wanted_tuple!(A, B, C, D, E);
wanted_tuple!(A, B, C, D, E, F);
wanted_tuple!(A, B, C, D, E, F, G);
wanted_tuple!(A, B, C, D, E, F, G, H);
