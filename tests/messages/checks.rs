//! What a station or a registry checks of a request before it acts on it, as the components'
//! documentation sets out, written once so that every path's tests check a request the same way:
//! that the request asks one thing, the authority it carries, what that authority reaches, the
//! document that follows a print, and where a registration asks to register the station; and what
//! a station checks of an application's endorsement of the registry that lists it.

use std::fmt;

use koinon_commons::{
    authority::{NotOpened, Party},
    codec::{DecodeError, ReadError, ReceivedComponents},
    devices::DeviceIds,
};

use super::*;

/// Why a station or a registry refuses a request: the code of the `Refusal` it answers with, and
/// why, for a test that expected another outcome to say.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refused {
    pub code: &'static str,
    pub why: String,
}

impl Refused {
    fn bad_request(why: impl fmt::Display) -> Self {
        Refused { code: Refusal::BAD_REQUEST, why: why.to_string() }
    }

    fn not_authorized(why: impl fmt::Display) -> Self {
        Refused { code: Refusal::NOT_AUTHORIZED, why: why.to_string() }
    }
}

impl fmt::Display for Refused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.why)
    }
}

impl Error for Refused {}

/// A component a request requires that is absent, or does not read: a request the receiver cannot
/// read as one it takes.
impl From<ReadError> for Refused {
    fn from(error: ReadError) -> Self {
        Refused::bad_request(error)
    }
}

/// A carrier whose signature does not verify carries no authority the station honours; one whose
/// envelope lists a component its reader does not understand is not understood; and one whose
/// envelope does not decode cannot be read.
impl From<NotOpened> for Refused {
    fn from(error: NotOpened) -> Self {
        match error {
            NotOpened::BadSignature(error) => Refused::not_authorized(error),
            NotOpened::Unreadable(error @ DecodeError::NotUnderstood { .. }) => {
                Refused { code: Refusal::NOT_UNDERSTOOD, why: error.to_string() }
            }
            NotOpened::Unreadable(error) => Refused::bad_request(error),
        }
    }
}

/// The request components a station takes.
const REQUESTS: &[MintedId] = &[Print::ID, Weigh::ID, ListDevices::ID, ClaimInvitation::ID, Retire::ID, VoidSlips::ID];

/// What `components` ask of the station: the one request component among them, since a request
/// asks one thing.
pub fn requested(components: &ReceivedComponents<'_>) -> Result<MintedId, Refused> {
    let asked: Vec<MintedId> = components.ids().filter(|id| REQUESTS.contains(id)).collect();
    match asked[..] {
        [request] => Ok(request),
        [] => Err(Refused::bad_request("the request asks nothing this station takes")),
        _ => Err(Refused::bad_request("the request asks more than one thing")),
    }
}

/// The request signed whole that an envelope carries, as a station takes it from the envelope
/// around it: the carrier alone listed there as a component a receiver must understand.
pub fn carried_signed_request(bytes: &[u8]) -> Result<SignedRequest, Refused> {
    let received = decoded(bytes, STATION_UNDERSTANDS);
    if received.must_understand() != [SignedRequest::ID] {
        return Err(Refused::bad_request("the envelope around a signed request lists more than its carrier"));
    }
    Ok(received.components().read()?)
}

/// The envelope of a request signed whole, opened as a station opens it at `NOW` by its clock,
/// whichever path brought it: signed by an application the station shares its devices with, its
/// signature verified, read as a reader of signed requests, with one request component and no
/// carrier inside, naming this station and no other, with an expiry neither passed nor further
/// off than a station honours. It gives the envelope and the station's approval of the signer.
pub fn opened_signed_request(request: &SignedRequest) -> Result<(ReceivedEnvelope<'_>, Approval), Refused> {
    let approval = approval(&request.signer).ok_or_else(|| Refused::not_authorized("the station shares with no such application"))?;
    let body = request.open(&Limits::default(), SIGNED_REQUEST_UNDERSTANDS)?;
    let components = body.components();
    if components.has::<PresentedSlip>() || components.has::<SignedRequest>() {
        return Err(Refused::bad_request("a signed request carries a carrier of its own"));
    }
    requested(&components)?;
    let (stations, expiry) = components.read::<(AtStations, Expiry)>()?;
    if stations.stations.keys() != [station()] {
        return Err(Refused::not_authorized("the request is signed for another station, or for more than one"));
    }
    if expiry.has_passed(at(NOW)) || SignedRequest::runs_too_long(&expiry, at(NOW)) {
        return Err(Refused::not_authorized("the request's signature has expired, or runs too long"));
    }
    Ok((body, approval))
}

/// What a station's person approved an application for.
pub enum Approval {
    /// Every device the station offers: an application that came through a registry.
    EveryDevice,
    /// The devices the person offered a computer they paired with the station directly.
    Offered(DeviceIds),
}

/// The station's approval of the application whose key is `signer`, when it shares its devices
/// with it: the application, through a registry, and a computer paired with it, offered the
/// printer alone.
pub fn approval(signer: &PublicKey) -> Option<Approval> {
    if *signer == key_of(&application()) {
        Some(Approval::EveryDevice)
    } else if *signer == key_of(&paired_computer()) {
        Some(Approval::Offered(DeviceIds::new(vec![printer_record().into()]).unwrap()))
    } else {
        None
    }
}

/// A permission slip as the station read it, once its checks of the slip passed: what it narrows
/// its application's approval to, and what it allows.
pub struct Slip {
    pub devices: Option<AtDevices>,
    pub allows: AllowsRequests,
}

/// What a station reads of the slip beside a request, at `NOW` by its clock, before it acts on the
/// request, where the slip's application's registry wrote the station's registration at
/// `REGISTERED` and last voided its slips at `voided`, if ever: signed by an application the
/// station shares its devices with, its signature verified, read as a reader of slips, for the
/// party the connection proves, issued no later than now and no earlier than the registration or
/// the voiding, running no longer than a slip may, unexpired, and naming this station. It gives
/// the slip and the station's approval of its application, which the slip narrows.
pub fn checked_slip(received: &ReceivedEnvelope<'_>, voided: Option<Timestamp>) -> Result<(Slip, Approval), Refused> {
    let presented: PresentedSlip = received.components().read()?;
    let approval = approval(&presented.signer).ok_or_else(|| Refused::not_authorized("the station shares with no such application"))?;
    let slip = presented.open(&Limits::default(), SLIP_UNDERSTANDS)?;
    let (body, slip_party, stations, devices, allows, expiry) =
        slip.components().read::<(PermissionSlip, Party, AtStations, Option<AtDevices>, AllowsRequests, Expiry)>()?;
    // The connection proves the party's key.
    if slip_party.key != party() {
        return Err(Refused::not_authorized("the slip is another party's"));
    }
    let honoured_from = voided.map_or(at(REGISTERED), |voided| voided.max(at(REGISTERED)));
    if body.is_issued_after(at(NOW)) || body.runs_too_long(&expiry) || body.is_issued_before(honoured_from) {
        return Err(Refused::not_authorized("the slip's moments are not ones a station honours"));
    }
    if expiry.has_passed(at(NOW)) {
        return Err(Refused::not_authorized("the slip has expired"));
    }
    if !stations.stations.contains(&station()) {
        return Err(Refused::not_authorized("the slip does not cover this station"));
    }
    Ok((Slip { devices, allows }, approval))
}

/// The hosting an application's endorsement vouches for, as a station takes it from the document
/// of the registry whose endpoint's key is `listed_by`, before it asks its person anything: its
/// signature verified, read as a reader of hostings, and taken only when it names that registry,
/// so that no registry lists an application under an endorsement of another registry.
pub fn taken_hosting(endorsement: &SignedHosting, listed_by: &PublicKey) -> Result<Hosting, Refused> {
    let hosting: Hosting = endorsement.open(&Limits::default(), HOSTING_UNDERSTANDS)?.components().read()?;
    if hosting.registry != *listed_by {
        return Err(Refused::not_authorized("the endorsement vouches for another registry's hosting"));
    }
    Ok(hosting)
}

/// The station's devices, each with the request that uses it: its printer, which prints, and its
/// scale, which weighs.
pub fn offered() -> [(EntityId, MintedId); 2] {
    [(printer_record(), Print::ID), (scale_record(), Weigh::ID)]
}

/// Whether a request under `approval`, and under `slip` when it came under one, reaches `device`,
/// which requests of the kind `kind` use, as `authority` sets out: the device is one the approval
/// lets the application use, and one the slip covers, of a kind whose requests the slip allows.
pub fn reaches(approval: &Approval, slip: Option<&Slip>, device: EntityId, kind: MintedId) -> bool {
    let approved = match approval {
        Approval::EveryDevice => true,
        Approval::Offered(devices) => devices.contains(&device.into()),
    };
    approved
        && slip.is_none_or(|slip| {
            slip.devices.as_ref().is_none_or(|narrowed| narrowed.devices.contains(&device.into())) && slip.allows.requests.contains(&kind)
        })
}

/// What a station takes of the stream after a print's envelope, by what it read of the print:
/// exactly as many bytes as its `DocumentFollows` gives, taken only when they hash to its
/// `PrintJob`'s hash. A stream that ends early, or a document that hashes to another, is refused.
pub fn taken_document<'s>(follows: &DocumentFollows, print_job: &PrintJob, stream: &'s [u8]) -> Result<&'s [u8], Refused> {
    let length =
        usize::try_from(follows.length.get()).map_err(|_| Refused::bad_request("the document is longer than this station takes"))?;
    let document = stream.get(..length).ok_or_else(|| Refused::bad_request("the stream ended before the document did"))?;
    if DocumentHash::of(document) != print_job.document_hash {
        return Err(Refused::bad_request("the document is not the one the print names"));
    }
    Ok(document)
}

/// Where a registration asks the registry to register the station.
#[derive(Debug, PartialEq)]
pub enum Place {
    /// A scope or sub-scope of the sign-in's answer.
    Listed(Placement),
    /// A first scope the registry makes, under this name.
    First(FirstScope),
}

/// Where a registration's `components` ask the registry to register the station, as the registry
/// reads them: a scope of the sign-in's answer, or a first scope, and never both.
pub fn place(components: &ReceivedComponents<'_>) -> Result<Place, Refused> {
    match components.read::<(Option<Placement>, Option<FirstScope>)>()? {
        (Some(placement), None) => Ok(Place::Listed(placement)),
        (None, Some(first)) => Ok(Place::First(first)),
        (Some(_), Some(_)) => Err(Refused::bad_request("the registration names two places")),
        (None, None) => Err(Refused::bad_request("the registration names no place")),
    }
}
