//! Building an envelope: its components added one by one, each property written through its
//! type's `Property` impl, then encoded in RFC 8949's core deterministic encoding (section 4.2.1),
//! so that the same components make the same bytes whoever sends them and in whatever order they
//! were added. Definite lengths, the shortest heads, and every map's keys in the bytewise order of
//! their encodings, which for ids is the order of their bytes; the must-understand list in that
//! order too, and the records in the order they were added.

use std::collections::{BTreeMap, BTreeSet};

use ankurah::{property::PropertyError, EntityId, Property, ValueType};

use super::{
    cbor::{self, ARRAY, MAP},
    value::{self, JsonWithParts},
    Minted, MintedId,
};

#[cfg(test)]
mod tests;

/// Why a component was not added to an envelope.
#[derive(Debug, thiserror::Error)]
pub enum EncodeError {
    #[error("component {component} was added twice")]
    DuplicateComponent { component: MintedId },
    #[error("component {component} writes property {property} twice")]
    DuplicateProperty { component: MintedId, property: MintedId },
    #[error("entity {entity} was given two records")]
    DuplicateRecord { entity: EntityId },
    /// The property's type did not convert it to a value, through its `Property` impl.
    #[error("property {property} of component {component} did not convert to a value: {error}")]
    Conversion { component: MintedId, property: MintedId, error: Box<PropertyError> },
    /// The property's type gave a value of another type than the one it declares.
    #[error("property {property} of component {component} declares {declared:?} values and gave a {produced:?}")]
    WrongValueType { component: MintedId, property: MintedId, declared: ValueType, produced: ValueType },
    #[error("property {property} of component {component} holds JSON with parts, an object or an array within an array, which no envelope carries")]
    JsonWithParts { component: MintedId, property: MintedId },
}

/// An envelope being built: its own components, the components a receiver must understand, and
/// the records it carries.
#[derive(Debug, Default)]
pub struct EnvelopeBuilder {
    own: Held,
    records: Vec<RecordBuilder>,
    /// The entities the records name, so that none is given two.
    entities: BTreeSet<EntityId>,
}

impl EnvelopeBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a component that a receiver which does not know it ignores.
    pub fn add<C: Minted>(&mut self, component: &C) -> Result<&mut Self, EncodeError> {
        self.own.add(component, false)?;
        Ok(self)
    }

    /// Adds a component that a receiver must understand: one that does not know it refuses the
    /// envelope, naming it. A restriction an older receiver must not drop, such as an expiry, is
    /// added so.
    pub fn add_must_understand<C: Minted>(&mut self, component: &C) -> Result<&mut Self, EncodeError> {
        self.own.add(component, true)?;
        Ok(self)
    }

    /// Adds a record, after those added before it. An entity has at most one record.
    pub fn add_record(&mut self, record: RecordBuilder) -> Result<&mut Self, EncodeError> {
        if let Some(entity) = record.entity {
            if !self.entities.insert(entity) {
                return Err(EncodeError::DuplicateRecord { entity });
            }
        }
        self.records.push(record);
        Ok(self)
    }

    /// The envelope's bytes.
    pub fn encode(&self) -> Vec<u8> {
        let must_understand: BTreeSet<MintedId> = self
            .records
            .iter()
            .map(|record| &record.held)
            .chain([&self.own])
            .flat_map(|held| held.must_understand.iter().copied())
            .collect();
        let records = self.records.iter().map(|record| 1 + 34 + record.held.len_bound()).sum::<usize>();
        let mut out = Vec::with_capacity(1 + 9 + 34 * must_understand.len() + self.own.len_bound() + 9 + records);
        cbor::write_head(&mut out, ARRAY, 3);
        cbor::write_head(&mut out, ARRAY, must_understand.len() as u64);
        for component in &must_understand {
            cbor::write_bytes(&mut out, component.as_bytes());
        }
        self.own.write(&mut out);
        cbor::write_head(&mut out, ARRAY, self.records.len() as u64);
        for record in &self.records {
            cbor::write_head(&mut out, ARRAY, 2);
            match record.entity {
                Some(entity) => cbor::write_bytes(&mut out, &entity.to_bytes()),
                None => out.push(cbor::NULL),
            }
            record.held.write(&mut out);
        }
        out
    }
}

/// A record being built for an envelope: an entity's components, under the entity's id, or under
/// none for a record that names no entity. A component a receiver must understand is so wherever
/// the envelope holds it.
#[derive(Debug)]
pub struct RecordBuilder {
    entity: Option<EntityId>,
    held: Held,
}

impl RecordBuilder {
    pub fn new(entity: Option<EntityId>) -> Self {
        RecordBuilder { entity, held: Held::default() }
    }

    /// Adds a component that a receiver which does not know it ignores.
    pub fn add<C: Minted>(&mut self, component: &C) -> Result<&mut Self, EncodeError> {
        self.held.add(component, false)?;
        Ok(self)
    }

    /// Adds a component that a receiver must understand, as [`EnvelopeBuilder::add_must_understand`]
    /// does.
    pub fn add_must_understand<C: Minted>(&mut self, component: &C) -> Result<&mut Self, EncodeError> {
        self.held.add(component, true)?;
        Ok(self)
    }
}

/// The components added to an envelope itself or to one of its records, each with its properties
/// encoded, in the order of their ids.
#[derive(Debug, Default)]
struct Held {
    components: BTreeMap<MintedId, Vec<(MintedId, Vec<u8>)>>,
    must_understand: BTreeSet<MintedId>,
}

impl Held {
    fn add<C: Minted>(&mut self, component: &C, must_understand: bool) -> Result<(), EncodeError> {
        if self.components.contains_key(&C::ID) {
            return Err(EncodeError::DuplicateComponent { component: C::ID });
        }
        let mut writer = PropertyWriter { component: C::ID, properties: Vec::new() };
        component.write_properties(&mut writer)?;
        writer.properties.sort_by_key(|&(property, _)| property);
        if let Some(pair) = writer.properties.windows(2).find(|pair| pair[0].0 == pair[1].0) {
            return Err(EncodeError::DuplicateProperty { component: C::ID, property: pair[0].0 });
        }
        self.components.insert(C::ID, writer.properties);
        if must_understand {
            self.must_understand.insert(C::ID);
        }
        Ok(())
    }

    /// Appends the map of these components.
    fn write(&self, out: &mut Vec<u8>) {
        cbor::write_head(out, MAP, self.components.len() as u64);
        for (component, properties) in &self.components {
            cbor::write_bytes(out, component.as_bytes());
            cbor::write_head(out, MAP, properties.len() as u64);
            for (property, value) in properties {
                cbor::write_bytes(out, property.as_bytes());
                out.extend_from_slice(value);
            }
        }
    }

    /// No fewer bytes than `write` appends: each id is 34, and no head more than 9.
    fn len_bound(&self) -> usize {
        9 + self
            .components
            .values()
            .map(|properties| 34 + 9 + properties.iter().map(|(_, value)| 34 + value.len()).sum::<usize>())
            .sum::<usize>()
    }
}

/// Where `#[derive(Minted)]` writes a component's properties, each under its minted id.
#[derive(Debug)]
pub struct PropertyWriter {
    component: MintedId,
    properties: Vec<(MintedId, Vec<u8>)>,
}

impl PropertyWriter {
    /// Writes `value` under `property`, converted through its type's `Property` impl as a stored
    /// record's property is: a value that converts to none, as an optional property's `None` does,
    /// leaves the property out.
    pub fn write<T: Property>(&mut self, property: MintedId, value: &T) -> Result<(), EncodeError> {
        let declared = const { value::value_type(T::VALUE_TYPE) };
        let component = self.component;
        let converted = value.into_value().map_err(|error| EncodeError::Conversion { component, property, error: Box::new(error) })?;
        let Some(value) = converted else { return Ok(()) };
        let produced = value::type_of(&value);
        if produced != declared {
            return Err(EncodeError::WrongValueType { component, property, declared, produced });
        }
        let mut bytes = Vec::new();
        value::encode(&value, &mut bytes).map_err(|JsonWithParts| EncodeError::JsonWithParts { component, property })?;
        self.properties.push((property, bytes));
        Ok(())
    }
}
