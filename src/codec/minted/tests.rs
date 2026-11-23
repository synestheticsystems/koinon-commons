//! What `#[derive(Minted)]` makes of the Koinon Commons' own models: they travel under the ids
//! Ankurah's derive reads from the same attributes, leave out the properties it marks optional when
//! those hold nothing, and read back as they were sent.

use ankurah::{property::Ref, EntityId, Model};
use ankurah_commons::dimension::Dimension;
use serde::Serialize;

use crate::{
    codec::{
        tests::{decoded, held, id, one},
        Minted, MintedId,
    },
    devices::{Device, Media, Printer, Scale, Station},
    mime_type::{MimeType, MimeTypes},
    nodes::NodeLink,
    public_key::PublicKey,
};

/// The ids Ankurah's descriptor holds for `C`: the model's, and each property's with whether it is
/// optional.
fn described<C: Model>() -> (MintedId, Vec<(MintedId, bool)>) {
    let descriptor = C::descriptor();
    let properties = descriptor.properties.iter().map(|property| (id(property.explicit_id.unwrap()), property.optional)).collect();
    (id(descriptor.explicit_id.unwrap()), properties)
}

/// Checks that `full`, with every property set, travels under the ids Ankurah's descriptor holds
/// and reads back as it was, and that `bare`, with every optional property unset, leaves out
/// exactly those.
fn travels_as_described<C: Minted + Serialize>(full: &C, bare: &C) {
    let name = C::descriptor().name;
    let (model, properties) = described::<C>();
    assert_eq!(C::ID, model, "{name}");
    let sorted = |mut ids: Vec<MintedId>| {
        ids.sort();
        ids
    };
    let all = sorted(properties.iter().map(|&(property, _)| property).collect());
    assert_eq!(held(&one(full)), [(model, all)], "{name}");
    let required = sorted(properties.iter().filter(|&&(_, optional)| !optional).map(|&(property, _)| property).collect());
    assert_eq!(held(&one(bare)), [(model, required)], "{name}");
    let bytes = one(full);
    let read: C = decoded(&bytes).components().read().unwrap();
    assert_eq!(serde_json::to_value(&read).unwrap(), serde_json::to_value(full).unwrap(), "{name}");
}

#[test]
fn every_model_of_the_commons_travels_under_the_ids_ankurahs_descriptor_holds() {
    let device = Device {
        station: Ref::from(EntityId::from_bytes([3; 32])),
        system_name: "Zebra_ZD421".into(),
        name: "Pack bench 3 labels".into(),
    };
    travels_as_described(&device, &device);
    let formats = ["application/pdf", "application/vnd.cups-raw"].map(|text| MimeType::new(text).unwrap());
    let printer = Printer { driver: "Zebra ZPL Label Printer".into(), document_formats: MimeTypes::new(formats.to_vec()).unwrap() };
    travels_as_described(&printer, &printer);
    let media = Media {
        name: "4x6".into(),
        width: Dimension::from_millimeters(101.6).unwrap(),
        height: Some(Dimension::from_millimeters(152.4).unwrap()),
    };
    travels_as_described(&media, &Media { height: None, ..media.clone() });
    travels_as_described(&Scale {}, &Scale {});
    let key = |seed| PublicKey::from(ed25519_dalek::SigningKey::from_bytes(&[seed; 32]).verifying_key());
    let station = Station { public_key: key(2), name: "Pack bench 3".into() };
    travels_as_described(&station, &station);
    let link = NodeLink { public_key: key(4), address: "https://registry.example.com".into(), name: "Registry".into() };
    travels_as_described(&link, &link);
}
