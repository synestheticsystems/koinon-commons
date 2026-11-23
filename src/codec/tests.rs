//! Components the codec's tests declare, with ids no mint gave them, one for each kind of property
//! a receiver meets, and the helpers its tests share. They belong to no namespace, so the guard on
//! the list of models passes over them.

use ankurah::{property::Ref, EntityId, Model};
use ankurah_commons::dimension::Dimension;

use super::{EnvelopeBuilder, Limits, Minted, MintedId, ReceivedEnvelope};
use crate::mime_type::{MimeType, MimeTypes};

/// What a person sees of something: its name, how many copies, and a note when there is one.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "tests.label")]
#[model(id = "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE")]
#[model(no_ffi)]
pub struct Label {
    #[active_type(LWW)]
    #[property(id = "ERERERERERERERERERERERERERERERERERERERERERE")]
    pub name: String,
    #[property(id = "EhISEhISEhISEhISEhISEhISEhISEhISEhISEhISEhI")]
    pub copies: i64,
    #[active_type(LWW)]
    #[property(id = "ExMTExMTExMTExMTExMTExMTExMTExMTExMTExMTExM")]
    pub note: Option<String>,
}

/// That something is marked, with nothing more to say. Its derives are listed in the other order
/// than the other components', which changes nothing.
#[derive(Minted, Model, Debug, Clone, PartialEq)]
#[model(label = "tests.marker")]
#[model(id = "AgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgI")]
#[model(no_ffi)]
pub struct Marker {}

/// A size, each side of which `Dimension` holds to being more than zero.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "tests.size")]
#[model(id = "AwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwM")]
#[model(no_ffi)]
pub struct Size {
    #[property(id = "MTExMTExMTExMTExMTExMTExMTExMTExMTExMTExMTE")]
    pub width: Dimension,
    #[active_type(LWW)]
    #[property(id = "MjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjI")]
    pub height: Option<Dimension>,
}

/// Until when something holds, in milliseconds: a restriction, which a sender adds as one a
/// receiver must understand.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "tests.deadline")]
#[model(id = "BAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQ")]
#[model(no_ffi)]
pub struct Deadline {
    #[property(id = "QUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUE")]
    pub until: i64,
}

/// The formats something takes: a list of MIME types, a json value.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "tests.formats")]
#[model(id = "BQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQU")]
#[model(no_ffi)]
pub struct Formats {
    #[active_type(LWW)]
    #[property(id = "UVFRUVFRUVFRUVFRUVFRUVFRUVFRUVFRUVFRUVFRUVE")]
    pub types: MimeTypes,
}

/// What something points at: an entity id.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "tests.pointer")]
#[model(id = "BgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgY")]
#[model(no_ffi)]
pub struct Pointer {
    #[property(id = "YWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWE")]
    pub target: Ref<Marker>,
}

/// Bytes carried whole: a binary value.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "tests.payload")]
#[model(id = "BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwc")]
#[model(no_ffi)]
pub struct Payload {
    #[property(id = "cXFxcXFxcXFxcXFxcXFxcXFxcXFxcXFxcXFxcXFxcXE")]
    pub bytes: Vec<u8>,
}

/// Every component above, as a receiver that understands them all names them.
pub(super) const UNDERSTOOD: &[MintedId] = &[Label::ID, Marker::ID, Size::ID, Deadline::ID, Formats::ID, Pointer::ID, Payload::ID];

pub(super) fn label() -> Label {
    Label { name: "Pack bench 3".into(), copies: 2, note: Some("Fragile".into()) }
}

pub(super) fn size() -> Size {
    Size { width: Dimension::from_millimeters(101.6).unwrap(), height: Some(Dimension::from_millimeters(152.4).unwrap()) }
}

pub(super) fn formats() -> Formats {
    let types = ["application/pdf", "application/vnd.cups-raw"].map(|text| MimeType::new(text).unwrap());
    Formats { types: MimeTypes::new(types.to_vec()).unwrap() }
}

pub(super) fn pointer() -> Pointer {
    Pointer { target: Ref::from(EntityId::from_bytes([9; 32])) }
}

pub(super) fn payload() -> Payload {
    Payload { bytes: (0..=255).collect() }
}

/// The id a minted id's text names.
pub(super) fn id(text: &str) -> MintedId {
    MintedId::from_bytes(EntityId::from_base64(text).unwrap().to_bytes())
}

/// The minted id `C`'s struct declares for the property of its field `field`.
pub(super) fn property<C: Minted>(field: &str) -> MintedId {
    let declared = C::descriptor().properties.iter().find(|property| property.field == field).unwrap();
    id(declared.explicit_id.unwrap())
}

/// An envelope that holds `component` alone.
pub(super) fn one<C: Minted>(component: &C) -> Vec<u8> {
    EnvelopeBuilder::new().add(component).unwrap().encode()
}

/// `bytes` as a receiver that understands every component above, within the default limits, reads
/// them.
pub(super) fn decoded(bytes: &[u8]) -> ReceivedEnvelope<'_> {
    ReceivedEnvelope::decode(bytes, &Limits::default(), UNDERSTOOD).unwrap()
}

/// The envelope's own components as `bytes` hold them, each by its id with its properties' ids, in
/// the order of the ids.
pub(super) fn held(bytes: &[u8]) -> Vec<(MintedId, Vec<MintedId>)> {
    let index = super::structure::index(bytes, &Limits::default(), UNDERSTOOD).unwrap();
    let properties = |component: &super::structure::ComponentEntry| {
        index.properties[component.properties.clone()].iter().map(|property| property.id).collect()
    };
    index.components[index.own.clone()].iter().map(|component| (component.id, properties(component))).collect()
}
