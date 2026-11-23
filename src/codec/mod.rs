//! Envelopes, in which Koinon nodes send each other components, so that nodes of different
//! versions understand each other wherever they share components. A newer sender adds a component
//! an older receiver does not know, a sender leaves out a component that does not concern it, and
//! every sender of one meaning sends the one component that means it.
//!
//! An envelope holds components, each a model whose struct declares its minted ids (`Minted`),
//! keyed by its id with its properties keyed by theirs; and records, each the components of one
//! entity. A receiver reads the components it understands and ignores the rest. It refuses an
//! envelope that lacks a component or a property it requires, and one that lists as
//! must-understand a component it does not understand, naming the component each time. A sender
//! lists so a component an older receiver must not drop, such as a restriction on what is asked.
//! A receiver understands a component only where it reads it: one that reads the envelope's own
//! components and none of its records, as the reader of a request or of what a carrier carries
//! does, decodes it with `ReceivedEnvelope::decode_own_components`, which refuses a component
//! listed so that a record holds. The codec's modules set out the encoding, each the part it
//! writes or reads, for an implementer in any language.
//!
//! A signature covers an envelope's bytes as they were encoded. A `ReceivedEnvelope` keeps the
//! bytes it was read from, and a `SealedEnvelope` carries an envelope's bytes untouched inside
//! another, so that no signature is checked over content decoded and encoded again.
//!
//! ```
//! use ankurah::{property::Ref, EntityId};
//! use koinon_commons::codec::{EnvelopeBuilder, Limits, Minted, ReceivedEnvelope, RecordBuilder};
//! use koinon_commons::devices::{Device, Printer};
//!
//! // A station lists a device it offers, as the record a store holds of it.
//! let device = Device {
//!     station: Ref::from(EntityId::from_bytes([3; 32])),
//!     system_name: "Zebra_ZD421".into(),
//!     name: "Pack bench 3 labels".into(),
//! };
//! let mut record = RecordBuilder::new(Some(EntityId::from_bytes([7; 32])));
//! record.add(&device)?;
//! let mut envelope = EnvelopeBuilder::new();
//! envelope.add_record(record)?;
//! let bytes = envelope.encode();
//!
//! // A receiver that understands devices and printers reads it, the printer being optional.
//! let received = ReceivedEnvelope::decode(&bytes, &Limits::default(), &[Device::ID, Printer::ID])?;
//! for record in received.records() {
//!     let (read, printer): (Device, Option<Printer>) = record.components().read()?;
//!     assert_eq!((read.name.as_str(), printer.is_none()), ("Pack bench 3 labels", true));
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod building;
mod cbor;
mod minted;
mod received;
mod sealed;
mod structure;
#[cfg(test)]
mod tests;
mod value;

pub use building::{EncodeError, EnvelopeBuilder, PropertyWriter, RecordBuilder};
pub use koinon_commons_derive::Minted;
pub use minted::{Minted, MintedId};
pub use received::{Properties, ReadError, ReceivedComponents, ReceivedEnvelope, ReceivedRecord, Wanted};
pub use sealed::SealedEnvelope;
pub use structure::{DecodeError, Limits};
pub use value::ValueProblem;

/// What the code `#[derive(Minted)]` generates names, so that it names nothing the codec's users
/// see and the codec can rearrange those freely.
#[doc(hidden)]
pub mod __private {
    pub use super::{EncodeError, Minted, MintedId, Properties, PropertyWriter, ReadError};
    pub use ankurah::Property;

    /// Fails to compile, where a property is declared, when its value type has no form in an
    /// envelope.
    pub const fn check_value_type(name: &str) {
        let _ = super::value::value_type(name);
    }
}
