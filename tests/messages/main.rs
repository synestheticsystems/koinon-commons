//! Every message on Pylon's iroh connections, built from the Koinon Commons' components, encoded
//! as an envelope, decoded as its receiver decodes it, and read back component by component: what
//! a station asks its registry, what a registry asks a station, what a party asks a station on the
//! direct path, and the replies to each; with the refusals the encoding promises.

mod direct;
mod refusals;
mod to_registry;
mod to_station;

use std::error::Error;

use ankurah::{EntityId, Model};
use ankurah_commons::{dimension::Dimension, property::Timestamp};
use ed25519_dalek::SigningKey;
use koinon_commons::{
    authority::{
        requests::{Retire, VoidSlips},
        AllowsRequests, AtDevices, AtStations, Expiry, ForApplication, PermissionSlip, PresentedSlip, Retirement, Signed, SignedRequest,
        SignedRetirement, SlipsVoided,
    },
    catalog::{requests::GetCatalog, PrinterFingerprint},
    codec::{EnvelopeBuilder, Limits, Minted, MintedId, ReceivedEnvelope, SealedEnvelope},
    devices::{requests::ListDevices, Device, Media, Printer, Scale, ScaleReading, Station},
    jobs::{
        requests::{Print, Weigh},
        Copies, DocumentFollows, DocumentHash, DocumentLength, ForDevice, JobAttempt, JobFailure, JobId, JobLabel, PrintJob,
    },
    mime_type::{MimeType, MimeTypes},
    nodes::{requests::PushEvents, Events},
    public_key::PublicKey,
    registry::{
        requests::{Hello, Register, SignIn, StopSharing},
        FirstScope, IdTokenSignIn, Placement, RegisteredFor, Registrable, Scope, ScopeAssignment, SignedInPerson,
    },
    replies::{Done, NotUnderstood, Refusal},
};
use sha2::{Digest, Sha256};

type Outcome = Result<(), Box<dyn Error>>;

/// Every component a station reads of a request, whichever path brought it, the authorities and
/// their restrictions among them.
const STATION_UNDERSTANDS: &[MintedId] = &[
    Print::ID,
    Weigh::ID,
    ListDevices::ID,
    Retire::ID,
    VoidSlips::ID,
    ForDevice::ID,
    JobId::ID,
    JobLabel::ID,
    PrintJob::ID,
    DocumentFollows::ID,
    SignedRequest::ID,
    PresentedSlip::ID,
    SignedRetirement::ID,
    PermissionSlip::ID,
    AtStations::ID,
    AtDevices::ID,
    AllowsRequests::ID,
    Expiry::ID,
    Retirement::ID,
    SlipsVoided::ID,
    ForApplication::ID,
];

/// Every component a registry reads of a station's request.
const REGISTRY_UNDERSTANDS: &[MintedId] = &[
    Hello::ID,
    SignIn::ID,
    Register::ID,
    PushEvents::ID,
    GetCatalog::ID,
    StopSharing::ID,
    IdTokenSignIn::ID,
    ScopeAssignment::ID,
    Placement::ID,
    FirstScope::ID,
    ForApplication::ID,
    Events::ID,
];

/// Every component a station, a registry or a party reads of a reply to its request.
const REPLIES_UNDERSTOOD: &[MintedId] = &[
    Done::ID,
    Refusal::ID,
    NotUnderstood::ID,
    RegisteredFor::ID,
    SignedRetirement::ID,
    SlipsVoided::ID,
    ForApplication::ID,
    SignedInPerson::ID,
    Scope::ID,
    Registrable::ID,
    Media::ID,
    PrinterFingerprint::ID,
    Device::ID,
    Printer::ID,
    Scale::ID,
    Station::ID,
    JobAttempt::ID,
    JobFailure::ID,
    ScaleReading::ID,
];

/// The moment the requests are made at, in milliseconds since 1970.
const NOW: i64 = 1_794_600_000_000;

const MINUTE: i64 = 60_000;

fn at(millis: i64) -> Timestamp {
    Timestamp::from_unix_millis(millis)
}

/// The secret key of the application whose authority the requests carry.
fn application() -> SigningKey {
    SigningKey::from_bytes(&[1; 32])
}

fn key(seed: u8) -> PublicKey {
    PublicKey::from(SigningKey::from_bytes(&[seed; 32]).verifying_key())
}

/// The station the requests are for.
fn station() -> PublicKey {
    key(2)
}

/// The party a permission slip is for.
fn party() -> PublicKey {
    key(3)
}

fn entity(byte: u8) -> EntityId {
    EntityId::from_bytes([byte; 32])
}

/// The station's own record, and its printer's and its scale's.
fn station_record() -> EntityId {
    entity(10)
}

fn printer_record() -> EntityId {
    entity(11)
}

fn scale_record() -> EntityId {
    entity(12)
}

/// `bytes` decoded as a receiver that understands `understood` decodes them, within the default
/// limits.
fn decoded<'a>(bytes: &'a [u8], understood: &[MintedId]) -> ReceivedEnvelope<'a> {
    ReceivedEnvelope::decode(bytes, &Limits::default(), understood).unwrap()
}

/// `body`, encoded, as the application's key signs it under the carrier `C`.
fn signed<C: Signed>(body: &EnvelopeBuilder) -> C {
    C::sign(SealedEnvelope::new(body.encode()), &application())
}

/// What the devices' components hold, so that one as sent and as read compare whole.
fn json<T: serde::Serialize>(component: &T) -> serde_json::Value {
    serde_json::to_value(component).unwrap()
}

/// The minted id that `C`'s struct declares for the property of its field `field`.
fn property<C: Minted>(field: &str) -> MintedId {
    let declared = C::descriptor().properties.iter().find(|property| property.field == field).unwrap();
    MintedId::from_bytes(EntityId::from_base64(declared.explicit_id.unwrap()).unwrap().to_bytes())
}

/// A label printed by its own language, ZPL, and every fact of a print of it but its authority.
struct LabelPrint {
    document: &'static [u8],
    job: JobId,
    device: ForDevice,
    label: JobLabel,
    print_job: PrintJob,
    follows: DocumentFollows,
}

fn label_print() -> LabelPrint {
    let document: &[u8] = b"^XA^FO50,50^ADN,36,20^FDOrder 1001^FS^XZ";
    LabelPrint {
        document,
        job: JobId { job: "order-1001-label".into() },
        device: ForDevice { device: printer_record().into() },
        label: JobLabel { name: "Order 1001".into(), asked_by: Some("packer-7".into()) },
        print_job: PrintJob {
            document_format: MimeType::new("application/x-zpl2").unwrap(),
            copies: Copies::new(2).unwrap(),
            document_hash: DocumentHash::from_bytes(Sha256::digest(document).into()),
        },
        follows: DocumentFollows { length: DocumentLength::new(document.len() as u64).unwrap() },
    }
}

impl LabelPrint {
    /// Adds the print's own components to `envelope`: the request, its facts, and the document's
    /// length, which changes how the stream is read.
    fn add_to(&self, envelope: &mut EnvelopeBuilder) -> Outcome {
        envelope
            .add_must_understand(&Print {})?
            .add(&self.job)?
            .add(&self.device)?
            .add(&self.label)?
            .add(&self.print_job)?
            .add_must_understand(&self.follows)?;
        Ok(())
    }

    /// Checks what a station read of the print against what was sent, and the document that
    /// follows the envelope against the hash and the length it read.
    fn read_back(&self, read: (Print, JobId, ForDevice, Option<JobLabel>, PrintJob, DocumentFollows)) {
        let (_, job, device, label, print_job, follows) = read;
        assert_eq!(
            (job, device, label, &print_job, &follows),
            (self.job.clone(), self.device.clone(), Some(self.label.clone()), &self.print_job, &self.follows)
        );
        assert_eq!(self.document.len() as u64, follows.length.get());
        assert_eq!(DocumentHash::from_bytes(Sha256::digest(self.document).into()), print_job.document_hash);
    }
}

/// A Zebra label printer, as its station offers it: its record, what its print system says of it,
/// and the stock a person said it is loaded with.
fn printer() -> (Device, Printer, Media) {
    let formats = ["application/vnd.cups-raw", "application/x-zpl2", "application/pdf"].map(|format| MimeType::new(format).unwrap());
    (
        Device { station: station_record().into(), system_name: "Zebra_ZD421".into(), name: "Pack bench 3 labels".into() },
        Printer { driver: "Zebra ZPL Label Printer".into(), document_formats: MimeTypes::new(formats.to_vec()).unwrap() },
        stock("4x6", 101.6, Some(152.4)),
    )
}

fn stock(name: &str, width: f64, height: Option<f64>) -> Media {
    let side = |millimeters| Dimension::from_millimeters(millimeters).unwrap();
    Media { name: name.into(), width: side(width), height: height.map(side) }
}

fn scale() -> Device {
    Device { station: station_record().into(), system_name: "SN 4411".into(), name: "Pack bench 3 scale".into() }
}

/// What a station answers a request with that asks for nothing back, read as the asker reads it.
fn assert_done(bytes: &[u8]) -> Outcome {
    let reply = decoded(bytes, REPLIES_UNDERSTOOD);
    let (refusal, done): (Option<Refusal>, Option<Done>) = reply.components().read()?;
    assert_eq!((refusal, done), (None, Some(Done {})));
    Ok(())
}

fn done() -> Vec<u8> {
    EnvelopeBuilder::new().add(&Done {}).unwrap().encode()
}
