//! What a station or a registry checks of a request before it acts on it, as the components'
//! documentation sets out, written once so that every path's tests check a request the same way:
//! that the request asks one thing, the authority it carries, what that authority reaches, the
//! document that follows a print, and where a registration asks to register the station; and what
//! a station checks of an application's endorsement of the registry that lists it. What the station
//! keeps that these consult is in `records`.

use std::fmt;

use koinon_commons::{
    authority::{NotOpened, Party},
    codec::{DecodeError, ReadError, ReceivedComponents},
};

use super::{
    records::{approval, is_blocked, Approval},
    *,
};

/// Why a station or a registry refuses a request: the code of the `Refusal` it answers with, and
/// why, which the refusal's words say; and for a refusal as `not_understood`, the components the
/// request listed as ones a receiver must understand that the receiver does not understand where
/// they stand, by their ids, which a `NotUnderstood` beside the refusal names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refused {
    pub code: &'static str,
    pub why: String,
    /// Empty but for a refusal as `not_understood`.
    pub not_understood: Vec<MintedId>,
}

impl Refused {
    pub fn bad_request(why: impl fmt::Display) -> Self {
        Refused { code: Refusal::BAD_REQUEST, why: why.to_string(), not_understood: Vec::new() }
    }

    pub fn not_authorized(why: impl fmt::Display) -> Self {
        Refused { code: Refusal::NOT_AUTHORIZED, why: why.to_string(), not_understood: Vec::new() }
    }

    pub fn not_understood(components: Vec<MintedId>, why: impl fmt::Display) -> Self {
        Refused { code: Refusal::NOT_UNDERSTOOD, why: why.to_string(), not_understood: components }
    }

    /// The reply that refuses the request, as the receiver encodes it: the `Refusal`, and beside a
    /// refusal as `not_understood` the `NotUnderstood` that names each component it did not
    /// understand, so that the sender learns them all in one reply.
    pub fn encode(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut reply = EnvelopeBuilder::new();
        reply.add(&Refusal { code: self.code.into(), message: self.why.clone() })?;
        if !self.not_understood.is_empty() {
            reply.add(&NotUnderstood { components: ComponentIds::new(self.not_understood.clone())? })?;
        }
        Ok(reply.encode())
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

/// An envelope that lists as ones a receiver must understand components this receiver does not
/// understand where they stand is not understood, naming them; one that does not decode cannot be
/// read.
impl From<DecodeError> for Refused {
    fn from(error: DecodeError) -> Self {
        match error {
            DecodeError::NotUnderstood { components } => {
                let why = format!("the envelope lists components this receiver does not understand: {}", ids_text(&components));
                Refused::not_understood(components, why)
            }
            error => Refused::bad_request(error),
        }
    }
}

/// A carrier whose signature does not verify carries no authority the station honours; one whose
/// envelope does not read is refused as any envelope that does not read is.
impl From<NotOpened> for Refused {
    fn from(error: NotOpened) -> Self {
        match error {
            NotOpened::BadSignature(error) => Refused::not_authorized(error),
            NotOpened::Unreadable(error) => Refused::from(error),
        }
    }
}

/// The ids of `components`, as a refusal's words name them.
fn ids_text(components: &[MintedId]) -> String {
    components.iter().map(MintedId::to_string).collect::<Vec<_>>().join(", ")
}

/// Whether `outcome` is a refusal as `not_authorized`.
pub fn not_authorized<T>(outcome: Result<T, Refused>) -> bool {
    outcome.is_err_and(|refused| refused.code == Refusal::NOT_AUTHORIZED)
}

/// The request components a station takes, each with the components beside it that it acts on,
/// those it requires and those it takes, as its documentation gives them.
const REQUESTS: &[(MintedId, &[MintedId])] = &[
    (Print::ID, &[ForDevice::ID, JobId::ID, JobLabel::ID, PrintJob::ID, DocumentFollows::ID]),
    (Weigh::ID, &[ForDevice::ID]),
    (ListDevices::ID, &[]),
    (ClaimInvitation::ID, &[InvitationClaim::ID]),
    (Retire::ID, &[SignedRetirement::ID]),
    (VoidSlips::ID, &[SlipsVoided::ID, ForApplication::ID]),
];

/// What `received` asks of the station: the one request component among its own components,
/// since a request asks one thing. Its envelope lists as ones a receiver must understand that
/// request, the components the request acts on, and those of `authority`, the components of its
/// authority that the station acts on, and no other: a component listed so that neither acts on,
/// such as a `DocumentFollows` beside a weigh, whose station reads no document after it, is one
/// the station would drop, and refuses the request as not understood.
pub fn requested(received: &ReceivedEnvelope<'_>, authority: &[MintedId]) -> Result<MintedId, Refused> {
    let components = received.components();
    let asked: Vec<&(MintedId, &[MintedId])> =
        components.ids().filter_map(|id| REQUESTS.iter().find(|(request, _)| *request == id)).collect();
    let (request, acted_on) = match asked[..] {
        [&asked] => asked,
        [] => return Err(Refused::bad_request("the request asks nothing this station takes")),
        _ => return Err(Refused::bad_request("the request asks more than one thing")),
    };
    let dropped: Vec<MintedId> = received
        .must_understand()
        .iter()
        .copied()
        .filter(|component| *component != request && !acted_on.contains(component) && !authority.contains(component))
        .collect();
    if !dropped.is_empty() {
        let why = format!("the request lists {} as ones it must understand, which it does not act on", ids_text(&dropped));
        return Err(Refused::not_understood(dropped, why));
    }
    Ok(request)
}

/// The request signed whole that an envelope carries, as a station takes it from the envelope
/// around it: the carrier alone listed there as a component a receiver must understand.
pub fn carried_signed_request(bytes: &[u8]) -> Result<SignedRequest, Refused> {
    let received = station_reads(bytes)?;
    if received.must_understand() != [SignedRequest::ID] {
        return Err(Refused::bad_request("the envelope around a signed request lists more than its carrier"));
    }
    Ok(received.components().read()?)
}

/// The path a request came by to the station.
pub enum Path {
    /// A party's own connection to the station, with no registry between them.
    Direct,
    /// The connection the station holds to a registry, whose endpoint's key the connection proves.
    Registry(PublicKey),
}

/// The envelope of a request signed whole, opened as a station opens it at `NOW` by its clock,
/// having come by `path`: signed by an application the station shares its devices with, and,
/// down a registry's connection, by one that came through that registry, since a registry's
/// gateway delivers jobs for its own applications alone; its signature verified, read as a reader
/// of signed requests, with one request component and no carrier inside, listing as ones a
/// receiver must understand only what that request and its restrictions act on, naming this
/// station and no other, with an expiry neither passed nor further off than a station honours. It
/// gives the envelope and the station's approval of the signer.
pub fn opened_signed_request<'r>(request: &'r SignedRequest, path: &Path) -> Result<(ReceivedEnvelope<'r>, Approval), Refused> {
    let approval = approval(&request.signer).ok_or_else(|| Refused::not_authorized("the station shares with no such application"))?;
    if let Path::Registry(registry) = path {
        if !approval.came_through(registry) {
            return Err(Refused::not_authorized("the request's application did not come through the registry that brought it"));
        }
    }
    let body = request.open(&Limits::default(), SIGNED_REQUEST_UNDERSTANDS)?;
    let components = body.components();
    if components.has::<PresentedSlip>() || components.has::<SignedRequest>() {
        return Err(Refused::bad_request("a signed request carries a carrier of its own"));
    }
    requested(&body, &[AtStations::ID, Expiry::ID])?;
    let (stations, expiry) = components.read::<(AtStations, Expiry)>()?;
    if stations.stations.keys() != [station()] {
        return Err(Refused::not_authorized("the request is signed for another station, or for more than one"));
    }
    if expiry.has_passed(at(NOW)) || SignedRequest::runs_too_long(&expiry, at(NOW)) {
        return Err(Refused::not_authorized("the request's signature has expired, or runs too long"));
    }
    Ok((body, approval))
}

/// A permission slip as the station read it, once its checks of the slip passed: what it narrows
/// its application's approval to, and what it allows.
pub struct Slip {
    pub devices: Option<AtDevices>,
    pub allows: AllowsRequests,
}

/// What a station reads of a request beside a permission slip, at `NOW` by its clock, before it
/// acts on it, where the connection that brought it proves the key `connection`, and the slip's
/// application's registry wrote the station's registration at `REGISTERED` and last voided its
/// slips at `voided`, if ever. The request asks one thing, and lists as ones a receiver must
/// understand only what that request and its slip act on. The slip is signed by an application
/// the station shares its devices with, its signature verified, read as a reader of slips, for
/// the party the connection proves, which the station's person has not blocked under the
/// application, issued no later than now and no earlier than the registration or the voiding,
/// expiring no earlier than its issue, running no longer than a slip may, unexpired, and naming
/// this station. It gives the slip and the station's approval of its application, which the slip
/// narrows.
pub fn under_slip(received: &ReceivedEnvelope<'_>, connection: &PublicKey, voided: Option<Timestamp>) -> Result<(Slip, Approval), Refused> {
    requested(received, &[PresentedSlip::ID])?;
    let presented: PresentedSlip = received.components().read()?;
    let approval = approval(&presented.signer).ok_or_else(|| Refused::not_authorized("the station shares with no such application"))?;
    let slip = presented.open(&Limits::default(), SLIP_UNDERSTANDS)?;
    let (body, slip_party, stations, devices, allows, expiry) =
        slip.components().read::<(PermissionSlip, Party, AtStations, Option<AtDevices>, AllowsRequests, Expiry)>()?;
    if slip_party.key != *connection {
        return Err(Refused::not_authorized("the slip is another party's"));
    }
    if is_blocked(&presented.signer, &slip_party.key) {
        return Err(Refused::not_authorized("the station's person blocked the slip's party"));
    }
    let honoured_from = voided.map_or(at(REGISTERED), |voided| voided.max(at(REGISTERED)));
    if body.is_issued_after(at(NOW))
        || body.expires_before_issue(&expiry)
        || body.runs_too_long(&expiry)
        || body.is_issued_before(honoured_from)
    {
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

/// The voiding of an application's slips that `received` asks a station to apply, as the station
/// takes it, having come by `path`: only down the connection of the registry the application came
/// through, since the voiding has its authority from that connection alone.
pub fn voiding(received: &ReceivedEnvelope<'_>, path: &Path) -> Result<(SlipsVoided, ForApplication), Refused> {
    requested(received, &[])?;
    let (_, voided, application) = received.components().read::<(VoidSlips, SlipsVoided, ForApplication)>()?;
    let from_its_registry = match path {
        Path::Registry(registry) => approval(&application.key).is_some_and(|approval| approval.came_through(registry)),
        Path::Direct => false,
    };
    if !from_its_registry {
        return Err(Refused::not_authorized("the voiding did not come from the registry its application came through"));
    }
    Ok((voided, application))
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

/// Whether a request under `approval`, and under `slip` when it came under one, reaches `device`,
/// which requests of the kind `kind` use, as `authority` sets out: the device is one the approval
/// lets the application use, and one the slip covers, of a kind whose requests the slip allows.
pub fn reaches(approval: &Approval, slip: Option<&Slip>, device: EntityId, kind: MintedId) -> bool {
    let approved = match approval {
        Approval::ThroughRegistry(_) => true,
        Approval::Paired(offered) => offered.contains(&device.into()),
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
