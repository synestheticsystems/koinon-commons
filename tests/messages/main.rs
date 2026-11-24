//! Every message of koinon/pylon/2 and of the direct path, built from the Koinon Commons'
//! components and the Ankurah Commons' `Request`, `Response` and `AuthToken`, sent through a stream
//! as one envelope and at most one attachment, and read back as its receiver reads it, by the
//! tuple of components it names: what a station asks its registry, what a registry asks a
//! station, what a party asks a station on the direct path, and the answers to each; with the
//! refusals the envelope and a station's checks of authority give.

mod direct;
mod refusals;
mod to_registry;
mod to_station;

use std::error::Error;

use ankurah::{
    envelope::{
        message::{self, ReceiveError},
        ContentRef, EntityRow, EntityRows, Envelope, EnvelopeComponent, Limits, ReadError, Signature,
    },
    property::Ref,
    EntityId,
};
use ankurah_commons::{
    dimension::Dimension,
    property::Timestamp,
    requests::{RequestId, Response},
};
use ed25519_dalek::{Signer, SigningKey};
use koinon_commons::{
    authority::{AllowsRequests, AtStations, Expiry, Party, PermissionSlip, SignedRequestCheck, SlipCheck},
    component_ids::ComponentIds,
    devices::{Device, Media, Printer, Scale, Station},
    jobs::{operations::DoPrint, operations::DoWeigh, Copies, ForDevice, JobLabel, PrintJob},
    mime_type::{MimeType, MimeTypes},
    name::Name,
    public_key::{PublicKey, PublicKeys},
    replies::{NotUnderstood, Refusal},
};

type Outcome = Result<(), Box<dyn Error>>;

/// The most bytes an envelope may take on the stream, which a receiver refuses beyond.
const ENVELOPE_LIMIT: u32 = 1 << 20;

/// The most bytes an attachment may take, which a station refuses beyond before it reads any.
const ATTACHMENT_LIMIT: u64 = 64 << 20;

/// The moment the requests are made at, in milliseconds since 1970.
const NOW: i64 = 1_794_600_000_000;

const MINUTE: i64 = 60_000;

/// When the application's registry wrote the station's registration for the application, by the
/// registry's clock: the earliest a slip of the application the station honours was issued.
const REGISTERED: i64 = NOW - 60 * MINUTE;

fn at(millis: i64) -> Timestamp {
    Timestamp::from_unix_millis(millis)
}

fn signing_key(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn key(seed: u8) -> PublicKey {
    PublicKey::from(signing_key(seed).verifying_key())
}

fn key_of(secret: &SigningKey) -> PublicKey {
    PublicKey::from(secret.verifying_key())
}

/// The secret key of the application whose authority the requests carry, which came to the
/// station through a registry.
fn application() -> SigningKey {
    signing_key(1)
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

/// `envelope` signed by `signer`, as an application signs a request or a slip.
fn signed(mut envelope: Envelope, signer: &SigningKey) -> Envelope {
    let identity = key_of(signer).identity();
    let signature = signer.sign(&envelope.signed_bytes(&identity));
    envelope.add_signature(&identity, &Signature::Ed25519(signature.to_bytes())).unwrap();
    envelope
}

/// The stream of one message, `envelope` and `attachment` when there is one, as its sender writes
/// it.
async fn sent(envelope: &Envelope, attachment: Option<&[u8]>) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut stream = Vec::new();
    message::write(&mut stream, envelope, attachment).await?;
    Ok(stream)
}

/// The envelope at the head of `stream`, as its receiver reads it, before it decides whether to
/// read anything more.
async fn envelope_of(stream: &mut &[u8]) -> Result<Envelope, ReceiveError> {
    message::read_envelope(stream, ENVELOPE_LIMIT, &Limits::default()).await
}

/// The attachment that follows the envelope on `stream`, as its receiver takes it once it is the
/// one `document` names, and nothing after it.
async fn attachment_of(stream: &mut &[u8], document: &ContentRef) -> Result<Vec<u8>, ReceiveError> {
    let mut spool = Vec::new();
    message::read_attachment(stream, document, ATTACHMENT_LIMIT, &mut spool).await?;
    Ok(spool)
}

/// The message `envelope` makes alone, read by its receiver, who takes no attachment after it.
async fn received(envelope: &Envelope) -> Result<Envelope, Box<dyn Error>> {
    let stream = sent(envelope, None).await?;
    let mut stream = stream.as_slice();
    let received = envelope_of(&mut stream).await?;
    message::read_end(&mut stream).await?;
    Ok(received)
}

/// The rows `rows` holds, each read as its reader reads it, within the default limits.
fn rows_of(rows: &EntityRows) -> Result<Vec<EntityRow>, Box<dyn Error>> {
    Ok(rows.read(&Limits::default())?)
}

/// What the station knows when it checks a slip the party presents on its own connection.
fn slip_check<'a>(application: &'a PublicKey, party: &'a PublicKey, station: &'a PublicKey) -> SlipCheck<'a> {
    SlipCheck { application, party, station, honoured_from: at(REGISTERED), now: at(NOW) }
}

/// What the station knows when it checks a request signed whole.
fn signed_request_check<'a>(application: &'a PublicKey, station: &'a PublicKey) -> SignedRequestCheck<'a> {
    SignedRequestCheck { application, station, now: at(NOW) }
}

/// A week's permission slip the application signed for the party, at this station, issued a minute
/// ago, for the operations whose tags' ids are `allows`.
fn weeks_slip(allows: &[EntityId]) -> Envelope {
    let slip = Envelope::new((
        PermissionSlip { issued: at(NOW - MINUTE), slip_id: entity(30) },
        Party { key: party(), name: Name::new("Warehouse label PC").unwrap() },
        AtStations { stations: PublicKeys::new(vec![station()]).unwrap() },
        AllowsRequests { requests: ComponentIds::new(allows.to_vec()).unwrap() },
        Expiry { expires: at(NOW + 7 * 24 * 60 * MINUTE) },
    ))
    .unwrap();
    signed(slip, &application())
}

/// For a request signed whole: this station alone, for the five minutes a registry's gateway gives
/// a job to reach a station.
fn this_station_for_five_minutes() -> (AtStations, Expiry) {
    (AtStations { stations: PublicKeys::new(vec![station()]).unwrap() }, Expiry { expires: at(NOW + 5 * MINUTE) })
}

/// A label printed in its own language, ZPL, and every fact of a print of it but its authority.
struct LabelPrint {
    document: &'static [u8],
    device: ForDevice,
    label: JobLabel,
    job: PrintJob,
}

fn label_print() -> LabelPrint {
    let document: &[u8] = b"^XA^FO50,50^ADN,36,20^FDOrder 1001^FS^XZ";
    LabelPrint {
        document,
        device: ForDevice { device: Ref::new(printer_record()) },
        label: JobLabel { name: "Order 1001".into(), asked_by: Some("packer-7".into()) },
        job: PrintJob {
            job_id: "order-1001-label".into(),
            document_format: MimeType::new("application/x-zpl2").unwrap(),
            copies: Copies::new(2).unwrap(),
            document: ContentRef::to(document),
        },
    }
}

/// A Zebra label printer, as its station offers it: its record, what its print system says of it,
/// and the stock a person said it is loaded with.
fn printer() -> (Device, Printer, Media) {
    let formats = ["application/vnd.cups-raw", "application/x-zpl2", "application/pdf"].map(|format| MimeType::new(format).unwrap());
    (
        Device { station: Ref::new(station_record()), system_name: "Zebra_ZD421".into(), name: "Pack bench 3 labels".into() },
        Printer { driver: "Zebra ZPL Label Printer".into(), document_formats: MimeTypes::new(formats.to_vec()).unwrap() },
        stock("4x6", 101.6, Some(152.4)),
    )
}

fn stock(name: &str, width: f64, height: Option<f64>) -> Media {
    let side = |millimeters| Dimension::from_millimeters(millimeters).unwrap();
    Media { name: name.into(), width: side(width), height: height.map(side) }
}

fn scale() -> Device {
    Device { station: Ref::new(station_record()), system_name: "SN 4411".into(), name: "Pack bench 3 scale".into() }
}

/// The station's own record.
fn station_itself() -> Station {
    Station { public_key: station(), name: "Pack bench 3".into() }
}

/// The response that refuses the request `request_id` because its receiver could not read it as
/// one it takes, as a receiver writes it: as `not_understood`, with a `NotUnderstood` that names
/// every component flagged so that it does not understand, so that the sender learns them all in
/// one answer; and as `bad_request` for a component or a property it lacks or cannot read.
fn refusing(request_id: RequestId, unread: &ReadError) -> Result<Envelope, Box<dyn Error>> {
    let message = unread.to_string();
    let refusal = match unread {
        ReadError::NotUnderstood { components } => (
            Response { request_id },
            Refusal { code: Refusal::NOT_UNDERSTOOD.into(), message },
            Some(NotUnderstood { components: ComponentIds::new(components.clone())? }),
        ),
        _ => (Response { request_id }, Refusal { code: Refusal::BAD_REQUEST.into(), message }, None),
    };
    Ok(Envelope::new(refusal)?)
}

/// The refusal a response carries, read as its asker reads a response, which tells a refusal from
/// an answer before it reads what an answer holds: the `Refusal`, with the `NotUnderstood` beside
/// it when there is one, once the response names the request `asked`.
fn refusal_of(response: &Envelope, asked: &RequestId) -> Result<(Refusal, Option<NotUnderstood>), Box<dyn Error>> {
    let (answered, refusal, not_understood) = response.get::<(Response, Option<Refusal>, Option<NotUnderstood>)>()?;
    assert_eq!(&answered.request_id, asked, "the response names the request it answers");
    Ok((refusal.ok_or("the response is an answer, not a refusal")?, not_understood))
}

/// The operations a station offers on each kind of device, for the listing a slip narrows: a
/// printer prints and a scale weighs.
fn operation_of(row: &EntityRow) -> Option<EntityId> {
    if row.has::<Printer>() {
        Some(DoPrint::ID)
    } else if row.has::<Scale>() {
        Some(DoWeigh::ID)
    } else {
        None
    }
}
