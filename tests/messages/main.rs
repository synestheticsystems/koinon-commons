//! Every message of koinon/pylon/2 and of the direct path, built from the Koinon Commons'
//! components, encoded as an envelope, decoded as its receiver decodes it, and read back component
//! by component: what a station asks its registry, what a registry asks a station, what a party
//! asks a station on the direct path, and the replies to each; with the refusals the encoding and a
//! station's checks of authority promise.

mod authorities;
mod checks;
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
        requests::{ClaimInvitation, Retire, VoidSlips},
        AllowsRequests, AtDevices, AtStations, Expiry, ForApplication, InvitationClaim, Party, PermissionSlip, PresentedSlip, Retirement,
        Signed, SignedRequest, SignedRetirement, SlipsVoided,
    },
    catalog::{requests::GetCatalog, PrinterFingerprint},
    codec::{EnvelopeBuilder, Limits, Minted, MintedId, ReceivedEnvelope, SealedEnvelope},
    component_ids::ComponentIds,
    devices::DeviceIds,
    devices::{requests::ListDevices, Device, Media, Printer, Scale, ScaleReading, Station},
    jobs::{
        requests::{Print, Weigh},
        Copies, DocumentFollows, DocumentHash, DocumentLength, ForDevice, JobAttempt, JobFailure, JobId, JobLabel, PrintJob,
    },
    mime_type::{MimeType, MimeTypes},
    name::Name,
    nodes::{requests::PushEvents, Events},
    public_key::{PublicKey, PublicKeys},
    registry::{
        requests::{Hello, Register, SignIn, StopSharing},
        FirstScope, IdTokenSignIn, Placement, Registrable, Registration, Scope, ScopeAssignment, SignedInPerson,
    },
    replies::{Done, NotUnderstood, Refusal},
};

type Outcome = Result<(), Box<dyn Error>>;

/// What a station reads of the envelope a request comes in, whichever path brought it: the
/// requests it takes, their facts, and the carriers of their authority. It reads what a carrier
/// carries as a reader of that kind of statement, with a list of its own.
const STATION_UNDERSTANDS: &[MintedId] = &[
    Print::ID,
    Weigh::ID,
    ListDevices::ID,
    ClaimInvitation::ID,
    Retire::ID,
    VoidSlips::ID,
    ForDevice::ID,
    JobId::ID,
    JobLabel::ID,
    PrintJob::ID,
    DocumentFollows::ID,
    InvitationClaim::ID,
    SlipsVoided::ID,
    ForApplication::ID,
    SignedRequest::ID,
    PresentedSlip::ID,
    SignedRetirement::ID,
];

/// What a station reads inside a request signed whole: the requests it takes so, their facts and
/// their restrictions; and the carriers, which it refuses there.
const SIGNED_REQUEST_UNDERSTANDS: &[MintedId] = &[
    Print::ID,
    Weigh::ID,
    ListDevices::ID,
    ForDevice::ID,
    JobId::ID,
    JobLabel::ID,
    PrintJob::ID,
    DocumentFollows::ID,
    AtStations::ID,
    Expiry::ID,
    SignedRequest::ID,
    PresentedSlip::ID,
];

/// What a station reads inside a permission slip: its body, its party and its restrictions.
const SLIP_UNDERSTANDS: &[MintedId] = &[PermissionSlip::ID, Party::ID, AtStations::ID, AtDevices::ID, AllowsRequests::ID, Expiry::ID];

/// What a station reads inside an application's retirement.
const RETIREMENT_UNDERSTANDS: &[MintedId] = &[Retirement::ID];

/// What a registry reads of a station's request.
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
    Registration::ID,
    Events::ID,
];

/// What a station, a registry or a party reads of a reply to its request.
const REPLIES_UNDERSTOOD: &[MintedId] = &[
    Done::ID,
    Refusal::ID,
    NotUnderstood::ID,
    Registration::ID,
    ForApplication::ID,
    SignedRetirement::ID,
    SlipsVoided::ID,
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

/// When the application's registry wrote the station's registration for the application, by the
/// registry's clock: the earliest a slip of the application the station honours was issued.
const REGISTERED: i64 = NOW - 60 * MINUTE;

fn at(millis: i64) -> Timestamp {
    Timestamp::from_unix_millis(millis)
}

/// The secret key of the application whose authority the requests carry, which came to the
/// station through a registry.
fn application() -> SigningKey {
    SigningKey::from_bytes(&[1; 32])
}

/// The secret key of a computer the station's person paired with the station directly, which is
/// the computer's application.
fn paired_computer() -> SigningKey {
    SigningKey::from_bytes(&[5; 32])
}

fn key(seed: u8) -> PublicKey {
    PublicKey::from(SigningKey::from_bytes(&[seed; 32]).verifying_key())
}

fn key_of(secret: &SigningKey) -> PublicKey {
    PublicKey::from(secret.verifying_key())
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
    signed_by(&application(), body)
}

/// `body`, encoded, as `key` signs it under the carrier `C`.
fn signed_by<C: Signed>(key: &SigningKey, body: &EnvelopeBuilder) -> C {
    C::sign(SealedEnvelope::new(body.encode()), key)
}

/// A permission slip the application signed for the party, at the station, issued at `issued` and
/// expiring at `expires`, for the requests `allows`, narrowed to `devices` when there are any.
fn presented_slip(issued: i64, expires: i64, allows: &[MintedId], devices: &[EntityId]) -> Result<PresentedSlip, Box<dyn Error>> {
    let mut envelope = EnvelopeBuilder::new();
    envelope
        .add_must_understand(&PermissionSlip { issued: at(issued), slip_id: entity(30) })?
        .add_must_understand(&Party { key: party(), name: Name::new("Warehouse label PC")? })?
        .add_must_understand(&AtStations { stations: PublicKeys::new(vec![station()])? })?
        .add_must_understand(&AllowsRequests { requests: ComponentIds::new(allows.to_vec())? })?
        .add_must_understand(&Expiry { expires: at(expires) })?;
    if !devices.is_empty() {
        envelope.add_must_understand(&AtDevices { devices: DeviceIds::new(devices.iter().map(|&device| device.into()).collect())? })?;
    }
    Ok(signed(&envelope))
}

/// A week's slip for prints and weighs, issued a minute ago, narrowed to `devices` when there are
/// any.
fn weeks_slip(devices: &[EntityId]) -> Result<PresentedSlip, Box<dyn Error>> {
    presented_slip(NOW - MINUTE, NOW + 7 * 24 * 60 * MINUTE, &[Print::ID, Weigh::ID], devices)
}

/// `components` in a request that `key` signs for the stations `stations`, expiring at `expires`.
fn signed_request_by(
    key: &SigningKey,
    stations: &[PublicKey],
    expires: i64,
    components: impl FnOnce(&mut EnvelopeBuilder) -> Outcome,
) -> Result<SignedRequest, Box<dyn Error>> {
    let mut body = EnvelopeBuilder::new();
    components(&mut body)?;
    body.add_must_understand(&AtStations { stations: PublicKeys::new(stations.to_vec())? })?
        .add_must_understand(&Expiry { expires: at(expires) })?;
    Ok(signed_by(key, &body))
}

/// `components` in a request the application signs for this station alone, for the five minutes
/// a registry's gateway gives a job to reach a station.
fn signed_request(components: impl FnOnce(&mut EnvelopeBuilder) -> Outcome) -> Result<SignedRequest, Box<dyn Error>> {
    signed_request_by(&application(), &[station()], NOW + 5 * MINUTE, components)
}

/// The envelope a signed request travels in: the carrier alone, listed so that a station too old
/// to check a signature refuses it rather than ignores it.
fn carrying(request: &SignedRequest) -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(EnvelopeBuilder::new().add_must_understand(request)?.encode())
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
            document_hash: DocumentHash::of(document),
        },
        follows: DocumentFollows { length: DocumentLength::new(document.len() as u64).unwrap() },
    }
}

/// A print as a station reads it from an envelope: the request, its facts, and how its document
/// follows.
type ReadPrint = (Print, JobId, ForDevice, Option<JobLabel>, PrintJob, DocumentFollows);

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

    /// Checks what a station read of the print against what was sent, and that the station takes
    /// from the stream, by what it read, the document the sender sent after the envelope.
    fn read_back(&self, read: ReadPrint) {
        let (_, job, device, label, print_job, follows) = read;
        assert_eq!(
            (&job, &device, &label, &print_job, &follows),
            (&self.job, &self.device, &Some(self.label.clone()), &self.print_job, &self.follows)
        );
        assert_eq!(checks::taken_document(&follows, &print_job, self.document), Ok(self.document));
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

/// The station's own record.
fn station_itself() -> Station {
    Station { public_key: station(), name: "Pack bench 3".into() }
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

/// The code of the refusal that `bytes` carry, read as the asker reads a reply.
fn refusal_code(bytes: &[u8]) -> Result<String, Box<dyn Error>> {
    Ok(decoded(bytes, REPLIES_UNDERSTOOD).components().read::<Refusal>()?.code)
}
