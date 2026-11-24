//! What a receiver refuses, and how the refusal reads to the sender: a request that holds a
//! component the receiver must understand and does not, one that lacks a component its operation
//! requires, one whose value its type refuses, and one that claims both kinds of authority; an
//! application's endorsement of an address that is no host name; and what a receiver reads past.

use ankurah::{
    envelope::{EnvelopeComponent, ReadError},
    Model,
};
use ankurah_commons::requests::{AuthToken, Request, RequestId, Response};
use koinon_commons::{
    authority::{slip_token, AllowsRequests, AtDevices, ForApplication, HostName, Hosting, InvitationClaim, Retirement, SlipsVoided},
    catalog::{operations::DoListCatalog, Catalog},
    devices::{operations::DoListDevices, DeviceListing},
    jobs::{
        operations::{DoPrint, DoWeigh},
        JobAttempt,
    },
    nodes::operations::DoImportEvents,
    registry::{
        operations::{DoEndRegistration, DoListRegistrations, DoRegister},
        ForGrant, IdTokenSignIn, KeptRegistrations, Registration, ScopeChoices,
    },
};

use super::*;

/// An operation a later version of the Koinon Commons adds, which this station does not know.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq)]
#[model(id = "CAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAg")]
#[model(no_ffi)]
#[envelope(must_understand)]
pub struct DoCut {}

/// What a later version of a sender adds beside a print that a station may do without.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq)]
#[model(id = "CQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQk")]
#[model(no_ffi)]
pub struct PrintPriority {
    #[envelope(number = 0)]
    pub rank: i64,
}

/// `PrintJob` as a sender in another language writes it, with its copies as a bare number, which
/// may be one that no `Copies` holds.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq)]
#[model(id = "nQF-f7gcJIaahDZ02CJzwtanB7_YAW1t9NK1E0Ng-Nw")]
#[model(no_ffi)]
pub struct PrintJobWithAnyCopies {
    #[active_type(LWW)]
    #[envelope(number = 0)]
    pub job_id: String,
    #[active_type(LWW)]
    #[envelope(number = 1)]
    pub document_format: MimeType,
    #[envelope(number = 2)]
    pub copies: i64,
    #[active_type(LWW)]
    #[envelope(number = 3)]
    pub document: ContentRef,
}

/// `Hosting` as a registry might write it, with its address as any text, which may be one that no
/// `HostName` holds.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq)]
#[model(id = "AX3KgpiVAyAjTj-_hzmYIUdNx9ZB2g4ndHP5O7JBnYY")]
#[model(no_ffi)]
#[envelope(must_understand)]
pub struct HostingAtAnyAddress {
    #[active_type(LWW)]
    #[envelope(number = 0)]
    pub registry: PublicKey,
    #[active_type(LWW)]
    #[envelope(number = 1)]
    pub name: Name,
    #[active_type(LWW)]
    #[envelope(number = 2)]
    pub address: String,
}

// So that every receiver that does not know one refuses the request, naming it, rather than act
// without it: each operation's tag or the component that names it, the token, a slip's body, its
// party and each restriction, an application's hosting, retirement and voiding, and what tells one
// registration from another. A listing, a reply's facts and a job's facts a receiver may do without.
#[test]
fn the_components_a_receiver_must_understand_are_flagged_so_and_no_others() {
    let flagged = [
        DoPrint::MUST_UNDERSTAND,
        DoWeigh::MUST_UNDERSTAND,
        DoListDevices::MUST_UNDERSTAND,
        DoListRegistrations::MUST_UNDERSTAND,
        DoRegister::MUST_UNDERSTAND,
        DoEndRegistration::MUST_UNDERSTAND,
        DoImportEvents::MUST_UNDERSTAND,
        DoListCatalog::MUST_UNDERSTAND,
        InvitationClaim::MUST_UNDERSTAND,
        IdTokenSignIn::MUST_UNDERSTAND,
        Retirement::MUST_UNDERSTAND,
        SlipsVoided::MUST_UNDERSTAND,
        AuthToken::MUST_UNDERSTAND,
        PermissionSlip::MUST_UNDERSTAND,
        Party::MUST_UNDERSTAND,
        AtStations::MUST_UNDERSTAND,
        AtDevices::MUST_UNDERSTAND,
        AllowsRequests::MUST_UNDERSTAND,
        Expiry::MUST_UNDERSTAND,
        Hosting::MUST_UNDERSTAND,
        ForApplication::MUST_UNDERSTAND,
        Registration::MUST_UNDERSTAND,
        ForGrant::MUST_UNDERSTAND,
    ];
    assert!(flagged.iter().all(|&flag| flag), "{flagged:?}");
    let unflagged = [
        Request::MUST_UNDERSTAND,
        Response::MUST_UNDERSTAND,
        ForDevice::MUST_UNDERSTAND,
        PrintJob::MUST_UNDERSTAND,
        JobLabel::MUST_UNDERSTAND,
        JobAttempt::MUST_UNDERSTAND,
        DeviceListing::MUST_UNDERSTAND,
        Catalog::MUST_UNDERSTAND,
        ScopeChoices::MUST_UNDERSTAND,
        KeptRegistrations::MUST_UNDERSTAND,
        Refusal::MUST_UNDERSTAND,
        NotUnderstood::MUST_UNDERSTAND,
    ];
    assert!(unflagged.iter().all(|&flag| !flag), "{unflagged:?}");
}

// A receiver cannot read the `Request` of a request that holds a component it must understand and
// does not: every read refuses, naming that component. So the refusal names every such component,
// and holds no `Response`, since the receiver cannot read the id it would name.
#[tokio::test]
async fn a_request_for_an_operation_the_station_does_not_know_is_refused_naming_it() -> Outcome {
    let request = Envelope::new((Request { request_id: RequestId::new() }, DoCut {}, ForDevice { device: Ref::new(printer_record()) }))?;
    let taken = received(&request).await?;
    assert!(taken.has::<Request>());
    let unread = taken.get::<Request>().unwrap_err();
    assert!(matches!(&unread, ReadError::NotUnderstood { components } if *components == [DoCut::ID]), "{unread}");
    let unread_as_a_weigh = taken.get::<(Request, DoWeigh, ForDevice)>().unwrap_err();
    assert!(matches!(&unread_as_a_weigh, ReadError::NotUnderstood { components } if *components == [DoCut::ID]), "{unread_as_a_weigh}");

    let ReadError::NotUnderstood { components } = unread else { unreachable!() };
    let refusal =
        Refusal { code: Refusal::NOT_UNDERSTOOD.into(), message: "the request holds components this station does not know".into() };
    let answer = Envelope::new((refusal, NotUnderstood { components: ComponentIds::new(components)? }))?;
    let (response, refused, not_understood) = received(&answer).await?.get::<(Option<Response>, Refusal, NotUnderstood)>()?;
    assert_eq!((response, refused.code.as_str()), (None, Refusal::NOT_UNDERSTOOD));
    assert_eq!(not_understood.components.ids(), [DoCut::ID]);
    Ok(())
}

#[tokio::test]
async fn a_print_without_the_device_it_is_for_is_refused_as_a_bad_request_naming_the_component() -> Outcome {
    let print = label_print();
    let asked = RequestId::new();
    let request = Envelope::new((Request { request_id: asked.clone() }, DoPrint {}, print.job.clone()))?;
    let stream = sent(&request, Some(print.document)).await?;
    let taken = envelope_of(&mut stream.as_slice()).await?;
    let unread = taken.get::<(Request, DoPrint, ForDevice, PrintJob, Option<JobLabel>)>().unwrap_err();
    assert!(matches!(&unread, ReadError::MissingComponent { component } if *component == ForDevice::ID), "{unread}");

    // The station reads the request's id beside the operation it recognised, which it understands.
    let (request, _) = taken.get::<(Request, DoPrint)>()?;
    let (refusal, not_understood) = refusal_of(&received(&refusing(request.request_id, &unread)?).await?, &asked)?;
    assert_eq!((refusal.code.as_str(), not_understood), (Refusal::BAD_REQUEST, None));
    assert!(refusal.message.contains(&ForDevice::ID.to_string()), "{}", refusal.message);
    Ok(())
}

// A value is read through its type's own rule, so that no sender, in any language, asks a printer
// for no copies.
#[tokio::test]
async fn a_print_job_whose_copies_break_their_rule_is_refused_naming_the_property() -> Outcome {
    let print = label_print();
    let asked = RequestId::new();
    let job = PrintJobWithAnyCopies {
        job_id: print.job.job_id.clone(),
        document_format: print.job.document_format.clone(),
        copies: 0,
        document: print.job.document,
    };
    let request = Envelope::new((Request { request_id: asked.clone() }, DoPrint {}, print.device.clone(), job))?;
    let taken = received(&request).await?;
    let unread = taken.get::<(Request, DoPrint, ForDevice, PrintJob, Option<JobLabel>)>().unwrap_err();
    assert!(matches!(&unread, ReadError::MalformedValue { component, number: 2, .. } if *component == PrintJob::ID), "{unread}");

    let (request, _) = taken.get::<(Request, DoPrint)>()?;
    let (refusal, _) = refusal_of(&received(&refusing(request.request_id, &unread)?).await?, &asked)?;
    assert_eq!(refusal.code, Refusal::BAD_REQUEST);
    Ok(())
}

// A request has its authority from a signature over it or from a slip it presents, never both: a
// reader of either kind finds the other's components, which it must understand and does not read.
#[tokio::test]
async fn a_request_signed_whole_that_also_presents_a_slip_is_read_as_neither() -> Outcome {
    let print = label_print();
    let (stations, expiry) = this_station_for_five_minutes();
    let request = Envelope::new((
        Request { request_id: RequestId::new() },
        DoPrint {},
        print.device.clone(),
        print.job.clone(),
        stations,
        expiry,
        slip_token(&weeks_slip(&[DoPrint::ID])),
    ))?;
    let taken = received(&signed(request, &application())).await?;
    let as_signed = taken.get::<(Request, DoPrint, ForDevice, PrintJob, Option<JobLabel>, AtStations, Expiry)>().unwrap_err();
    assert!(matches!(&as_signed, ReadError::NotUnderstood { components } if *components == [AuthToken::ID]), "{as_signed}");
    let under_slip = taken.get::<(Request, DoPrint, ForDevice, PrintJob, Option<JobLabel>, AuthToken)>().unwrap_err();
    let ReadError::NotUnderstood { mut components } = under_slip else { panic!("{under_slip}") };
    components.sort();
    let mut restrictions = vec![AtStations::ID, Expiry::ID];
    restrictions.sort();
    assert_eq!(components, restrictions);
    Ok(())
}

// So that a newer sender adds what an older station can do without, and the station still acts.
#[tokio::test]
async fn a_component_a_station_need_not_understand_is_read_past() -> Outcome {
    let print = label_print();
    let request = Envelope::new((
        Request { request_id: RequestId::new() },
        DoPrint {},
        print.device.clone(),
        print.job.clone(),
        PrintPriority { rank: 1 },
    ))?;
    let stream = sent(&request, Some(print.document)).await?;
    let mut stream = stream.as_slice();
    let (_, _, device, job, label) = envelope_of(&mut stream).await?.get::<(Request, DoPrint, ForDevice, PrintJob, Option<JobLabel>)>()?;
    assert_eq!((device, &job, label), (print.device, &print.job, None));
    assert_eq!(attachment_of(&mut stream, &job.document).await?, print.document);
    Ok(())
}

// An application's endorsement of a registry's hosting of it is an envelope of the hosting alone,
// signed by the application's key, which a station reads from the registry's document as its exact
// bytes; one whose address is no host name in the one form a host name is written in does not
// read, naming the property, so that a station never compares its person's address with another
// writing of it.
#[test]
fn an_endorsement_reads_only_with_an_address_that_is_a_host_name() -> Outcome {
    let registry = key(6);
    let hosting = Hosting { registry, name: Name::new("Acme")?, address: HostName::new("acme.example.com")? };
    let endorsement = signed(Envelope::new(hosting.clone())?, &application());
    let read = Envelope::decode(endorsement.bytes().clone(), &Limits::default())?;
    assert!(read.is_signed_by(&key_of(&application()).identity()));
    assert_eq!(read.get::<Hosting>()?, hosting);

    let written = HostingAtAnyAddress { registry, name: Name::new("Acme")?, address: "Acme.example.com".into() };
    let read = Envelope::decode(signed(Envelope::new(written)?, &application()).bytes().clone(), &Limits::default())?;
    let unread = read.get::<Hosting>().unwrap_err();
    assert!(matches!(&unread, ReadError::MalformedValue { component, number: 2, .. } if *component == Hosting::ID), "{unread}");
    Ok(())
}
