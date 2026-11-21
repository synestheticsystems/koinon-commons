//! The one pass over a received envelope that checks every item of it before any component is
//! read, and indexes where each component and each of its properties lie in the bytes, which stay
//! as they were received.
//!
//! The pass holds every item to the form every envelope takes, the items of components the
//! receiver does not know among them, so that an envelope one node accepts no other refuses for
//! its form: the items `cbor` reads, with no map that holds a key twice and no map key that is an
//! array or a map, within the receiver's limits. Keys may come in any order, and heads and floats
//! may be longer than they need to be. Each item takes at least one byte of what remains and is
//! counted, so a declared length or count sets no memory aside.

use std::ops::Range;

use ankurah::EntityId;

use super::{
    cbor::{Item, Reader},
    MintedId,
};

#[cfg(test)]
mod tests;

/// How much of an envelope a receiver reads before it refuses it, against a sender that would make
/// it spend memory or stack. The frame that carries an envelope bounds its length before it is
/// read; these bound what the length holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// How deeply items may nest, the envelope's own array counting as the first level. The
    /// envelope takes three levels down to a property of one of its own components, and a record
    /// two more, so a value nests within what remains. The check takes one frame of the stack per
    /// level, so this bounds the stack too.
    pub max_depth: usize,
    /// How many items the envelope may hold in all, every key, value and element counting as one.
    pub max_items: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Limits { max_depth: 32, max_items: 65_536 }
    }
}

/// Why bytes are not an envelope this receiver takes, found before any component is read. Every
/// error but the last two says at which byte the trouble lies.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DecodeError {
    #[error("the envelope ends inside the item that begins at byte {at}")]
    Truncated { at: usize },
    #[error("bytes follow the envelope's end, at byte {at}")]
    TrailingBytes { at: usize },
    #[error("the item at byte {at} is {found}, where an envelope holds {expected}")]
    NotAnEnvelope { at: usize, expected: &'static str, found: &'static str },
    #[error("the item at byte {at} is {what}, which no envelope holds")]
    Excluded { at: usize, what: &'static str },
    #[error("the text string at byte {at} is not UTF-8")]
    NotUtf8 { at: usize },
    #[error("the item at byte {at} nests deeper than {limit} levels")]
    TooDeep { at: usize, limit: usize },
    #[error("the item at byte {at} is one more than the {limit} items an envelope may hold")]
    TooManyItems { at: usize, limit: usize },
    #[error("component {component} is held twice, the second time at byte {at}")]
    DuplicateComponent { component: MintedId, at: usize },
    #[error("component {component} holds property {property} twice, the second time at byte {at}")]
    DuplicateProperty { component: MintedId, property: MintedId, at: usize },
    #[error("a map in property {property} of component {component} holds a key twice, the second time at byte {at}")]
    DuplicateKey { component: MintedId, property: MintedId, at: usize },
    #[error("entity {entity} has two records, the second at byte {at}")]
    DuplicateRecord { entity: EntityId, at: usize },
    #[error("component {component} is listed twice as one a receiver must understand, the second time at byte {at}")]
    MustUnderstandTwice { component: MintedId, at: usize },
    #[error("component {component} is listed as one a receiver must understand, and the envelope does not hold it")]
    MustUnderstandAbsent { component: MintedId },
    /// The envelope lists as must-understand components this receiver does not understand, named
    /// all at once, so that a sender learns in one answer everything to leave out.
    #[error("the envelope lists as ones a receiver must understand components this receiver does not understand: {}", listed(components))]
    NotUnderstood { components: Vec<MintedId> },
}

fn listed(components: &[MintedId]) -> String {
    components.iter().map(MintedId::to_string).collect::<Vec<_>>().join(", ")
}

/// Where a received envelope's parts lie: the components it lists as must-understand, its own
/// components, and its records in the sender's order.
#[derive(Debug, Default)]
pub(crate) struct Index {
    /// In the order of their ids.
    pub must_understand: Vec<MintedId>,
    /// The envelope's own components, then each record's, each run in the order of their ids.
    pub components: Vec<ComponentEntry>,
    /// Each component's properties, each run in the order of their ids.
    pub properties: Vec<PropertyEntry>,
    /// The envelope's own components, in `components`.
    pub own: Range<usize>,
    pub records: Vec<RecordEntry>,
}

#[derive(Debug)]
pub(crate) struct ComponentEntry {
    pub id: MintedId,
    /// Where its id begins, which an error names.
    at: usize,
    /// Its properties, in [`Index::properties`].
    pub properties: Range<usize>,
}

#[derive(Debug)]
pub(crate) struct PropertyEntry {
    pub id: MintedId,
    /// Where its id begins, which an error names.
    at: usize,
    /// Where its value's one item lies in the envelope.
    pub value: Range<usize>,
}

#[derive(Debug)]
pub(crate) struct RecordEntry {
    pub entity: Option<EntityId>,
    /// Its components, in [`Index::components`].
    pub components: Range<usize>,
}

const ENVELOPE: &str = "an array of three: the components a receiver must understand, the components, and the records";
const LISTED: &str = "an array of component ids";
const COMPONENTS: &str = "a map from component id to properties";
const PROPERTIES: &str = "a map from property id to value";
const RECORDS: &str = "an array of records";
const RECORD: &str = "an array of two: an entity id or null, then the record's components";
const ID: &str = "a minted id: a byte string of 32 bytes";
const ENTITY: &str = "an entity id, a byte string of 32 bytes, or null";

/// Checks every item of `bytes` within `limits`, and indexes the envelope they hold, which may
/// list as must-understand only components in `understood`.
pub(crate) fn index(bytes: &[u8], limits: &Limits, understood: &[MintedId]) -> Result<Index, DecodeError> {
    let mut walk = Walk { reader: Reader::new(bytes), limits: *limits, items: 0, within: None };
    let mut index = Index::default();

    walk.array(1, ENVELOPE, Some(3))?;
    let mut listed = Vec::new();
    for _ in 0..walk.array(2, LISTED, None)? {
        listed.push(walk.id(3)?);
    }
    index.own = walk.components(2, &mut index)?;
    let mut entities = Vec::new();
    for _ in 0..walk.array(2, RECORDS, None)? {
        walk.array(3, RECORD, Some(2))?;
        let (at, entity) = walk.entity(4)?;
        if let Some(entity) = entity {
            entities.push((entity, at));
        }
        let components = walk.components(4, &mut index)?;
        index.records.push(RecordEntry { entity, components });
    }
    let end = walk.reader.position();
    if end != bytes.len() {
        return Err(DecodeError::TrailingBytes { at: end });
    }

    if let Some((entity, at)) = twice(&mut entities, |&entity| entity) {
        return Err(DecodeError::DuplicateRecord { entity, at });
    }
    if let Some((component, at)) = twice(&mut listed, |&listing| listing) {
        return Err(DecodeError::MustUnderstandTwice { component, at });
    }
    index.must_understand = listed.into_iter().map(|(component, _)| component).collect();
    if let Some(&component) =
        index.must_understand.iter().find(|component| !index.components.iter().any(|held| held.id == **component))
    {
        return Err(DecodeError::MustUnderstandAbsent { component });
    }
    let unknown: Vec<MintedId> = index.must_understand.iter().copied().filter(|component| !understood.contains(component)).collect();
    if !unknown.is_empty() {
        return Err(DecodeError::NotUnderstood { components: unknown });
    }
    Ok(index)
}

/// Sorts `entries` by their keys, each given with where the entry begins by `keyed`, and gives a
/// key two entries share, with where the later one begins.
fn twice<T, K: Ord + Copy>(entries: &mut [T], keyed: impl Fn(&T) -> (K, usize)) -> Option<(K, usize)> {
    entries.sort_by_key(&keyed);
    entries.windows(2).map(|pair| (keyed(&pair[0]).0, keyed(&pair[1]))).find(|(first, second)| *first == second.0).map(|(_, second)| second)
}

struct Walk<'b> {
    reader: Reader<'b>,
    limits: Limits,
    items: usize,
    /// The component and property whose value is being walked, which a key held twice names.
    within: Option<(MintedId, MintedId)>,
}

/// A map key, compared for a key held twice by the value it names rather than the bytes it takes:
/// an integer in a longer head, or a float in a wider one, is the same key.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Key<'b> {
    Integer(i128),
    Float(u64),
    Bytes(&'b [u8]),
    Text(&'b str),
    Bool(bool),
    Null,
}

impl<'b> Walk<'b> {
    /// Reads the next item, at `depth`, and counts it against the limits.
    fn next(&mut self, depth: usize) -> Result<(usize, Item<'b>), DecodeError> {
        let at = self.reader.position();
        if depth > self.limits.max_depth {
            return Err(DecodeError::TooDeep { at, limit: self.limits.max_depth });
        }
        self.items += 1;
        if self.items > self.limits.max_items {
            return Err(DecodeError::TooManyItems { at, limit: self.limits.max_items });
        }
        Ok((at, self.reader.item()?))
    }

    /// Reads the head of one of the envelope's arrays, of `len` elements when it says, and gives
    /// its length.
    fn array(&mut self, depth: usize, expected: &'static str, len: Option<u64>) -> Result<u64, DecodeError> {
        match self.next(depth)? {
            (at, Item::Array(found)) if len.is_some_and(|len| len != found) => {
                Err(DecodeError::NotAnEnvelope { at, expected, found: "an array of another length" })
            }
            (_, Item::Array(found)) => Ok(found),
            (at, other) => Err(DecodeError::NotAnEnvelope { at, expected, found: other.kind() }),
        }
    }

    fn map(&mut self, depth: usize, expected: &'static str) -> Result<u64, DecodeError> {
        match self.next(depth)? {
            (_, Item::Map(pairs)) => Ok(pairs),
            (at, other) => Err(DecodeError::NotAnEnvelope { at, expected, found: other.kind() }),
        }
    }

    /// Reads a component's or a property's id, and gives it with where it begins.
    fn id(&mut self, depth: usize) -> Result<(MintedId, usize), DecodeError> {
        match self.next(depth)? {
            (at, Item::Bytes(bytes)) => bytes
                .try_into()
                .map(|id| (MintedId::from_bytes(id), at))
                .map_err(|_| DecodeError::NotAnEnvelope { at, expected: ID, found: "a byte string of another length" }),
            (at, other) => Err(DecodeError::NotAnEnvelope { at, expected: ID, found: other.kind() }),
        }
    }

    /// Reads a record's entity id, or the null of a record that names none.
    fn entity(&mut self, depth: usize) -> Result<(usize, Option<EntityId>), DecodeError> {
        match self.next(depth)? {
            (at, Item::Null) => Ok((at, None)),
            (at, Item::Bytes(bytes)) => bytes
                .try_into()
                .map(|id| (at, Some(EntityId::from_bytes(id))))
                .map_err(|_| DecodeError::NotAnEnvelope { at, expected: ENTITY, found: "a byte string of another length" }),
            (at, other) => Err(DecodeError::NotAnEnvelope { at, expected: ENTITY, found: other.kind() }),
        }
    }

    /// Walks a map of components whose head is at `depth`, adds each component and its properties
    /// to `index`, and gives where the components lie in it.
    fn components(&mut self, depth: usize, index: &mut Index) -> Result<Range<usize>, DecodeError> {
        let first = index.components.len();
        for _ in 0..self.map(depth, COMPONENTS)? {
            let (component, at) = self.id(depth + 1)?;
            let start = index.properties.len();
            for _ in 0..self.map(depth + 1, PROPERTIES)? {
                let (property, at) = self.id(depth + 2)?;
                self.within = Some((component, property));
                let value = self.reader.position();
                self.value(depth + 2)?;
                index.properties.push(PropertyEntry { id: property, at, value: value..self.reader.position() });
            }
            if let Some((property, at)) = twice(&mut index.properties[start..], |property| (property.id, property.at)) {
                return Err(DecodeError::DuplicateProperty { component, property, at });
            }
            index.components.push(ComponentEntry { id: component, at, properties: start..index.properties.len() });
        }
        if let Some((component, at)) = twice(&mut index.components[first..], |component| (component.id, component.at)) {
            return Err(DecodeError::DuplicateComponent { component, at });
        }
        Ok(first..index.components.len())
    }

    /// Walks one item of a property's value at `depth`, and every item it holds.
    fn value(&mut self, depth: usize) -> Result<(), DecodeError> {
        match self.next(depth)?.1 {
            Item::Array(len) => {
                for _ in 0..len {
                    self.value(depth + 1)?;
                }
            }
            Item::Map(pairs) => {
                let mut keys = Vec::new();
                for _ in 0..pairs {
                    let (at, key) = self.next(depth + 1)?;
                    let key = match key {
                        Item::Integer(number) => Key::Integer(number),
                        Item::Float(number) => Key::Float(number.to_bits()),
                        Item::Bytes(bytes) => Key::Bytes(bytes),
                        Item::Text(text) => Key::Text(text),
                        Item::Bool(truth) => Key::Bool(truth),
                        Item::Null => Key::Null,
                        Item::Array(_) | Item::Map(_) => {
                            return Err(DecodeError::Excluded { at, what: "a map key that is an array or a map" })
                        }
                    };
                    keys.push((key, at));
                    self.value(depth + 1)?;
                }
                if let Some((_, at)) = twice(&mut keys, |&key| key) {
                    let (component, property) = self.within.expect("a map in an envelope lies within a property's value");
                    return Err(DecodeError::DuplicateKey { component, property, at });
                }
            }
            _ => {}
        }
        Ok(())
    }
}
