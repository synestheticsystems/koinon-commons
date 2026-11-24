//! What a station asks its registry, down the connection it holds to it, and how the registry
//! answers: the registrations it keeps, a person's sign-in, a registration, the end of one, the
//! station's events, and the catalog.

use ankurah::{envelope::EntityRows, property::Ref};
use ankurah_commons::requests::{Request, RequestId, Response};
use koinon_commons::{
    authority::{ForApplication, SlipsVoided},
    catalog::{operations::DoListCatalog, Catalog, PrinterFingerprint},
    devices::Media,
    nodes::{operations::DoImportEvents, Events},
    registry::{
        operations::{DoEndRegistration, DoListRegistrations, DoRegister},
        FirstScope, ForGrant, IdTokenSignIn, KeptRegistrations, Placement, Registrable, Registration, Scope, ScopeAssignment, ScopeChoices,
        SignedInPerson,
    },
};

use super::*;

fn for_application() -> ForApplication {
    ForApplication { key: key_of(&application()) }
}

fn registration(grant: u8, registered: i64) -> Registration {
    Registration { grant: Ref::new(entity(grant)), registered: at(registered) }
}

#[tokio::test]
async fn a_list_of_registrations_is_answered_with_each_registration_and_its_applications_latest_voiding() -> Outcome {
    let asked = RequestId::new();
    let request = received(&Envelope::new((Request { request_id: asked.clone() }, DoListRegistrations {}))?).await?;
    let (request, _) = request.get::<(Request, DoListRegistrations)>()?;

    let other_application = ForApplication { key: key(7) };
    let mut registrations = EntityRows::builder();
    registrations.push(None, (registration(40, REGISTERED), for_application()))?;
    registrations.push(None, (registration(41, REGISTERED), other_application.clone(), SlipsVoided { before: at(NOW - MINUTE) }))?;
    let answer = Envelope::new((Response { request_id: request.request_id }, KeptRegistrations { registrations: registrations.build() }))?;

    let (response, kept) = received(&answer).await?.get::<(Response, KeptRegistrations)>()?;
    assert_eq!(response.request_id, asked);
    let read = rows_of(&kept.registrations)?
        .iter()
        .map(|row| row.get::<(Registration, ForApplication, Option<SlipsVoided>)>())
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(
        read,
        [
            (registration(40, REGISTERED), for_application(), None),
            (registration(41, REGISTERED), other_application, Some(SlipsVoided { before: at(NOW - MINUTE) })),
        ]
    );
    Ok(())
}

#[tokio::test]
async fn a_sign_in_is_answered_with_the_person_and_the_scopes_they_may_register_in() -> Outcome {
    let sign_in = IdTokenSignIn {
        id_token: "eyJhbGciOiJSUzI1NiJ9.eyJzdWIiOiIxIn0.c2ln".into(),
        nonce_seed: "kV3pQ8sN1xR6tB0yW4zH7mC2fJ9dL5gA8eU3oI1nT6c".into(),
    };
    let asked = RequestId::new();
    let request = received(&Envelope::new((Request { request_id: asked.clone() }, sign_in.clone()))?).await?;
    let (request, taken) = request.get::<(Request, IdTokenSignIn)>()?;
    assert_eq!(taken, sign_in);

    // The person may register in Acme as a whole, and in one sub-scope of Globex alone.
    let acme = Scope { name: "Acme".into(), parent: None };
    let globex = Scope { name: "Globex".into(), parent: None };
    let globex_east = Scope { name: "Globex East".into(), parent: Some(Ref::new(entity(51))) };
    let mut scopes = EntityRows::builder();
    scopes.push(Some(entity(50)), (acme.clone(), Registrable {}))?;
    scopes.push(Some(entity(51)), globex.clone())?;
    scopes.push(Some(entity(52)), (globex_east.clone(), Registrable {}))?;
    let person = SignedInPerson { name: Some("Ada Lovelace".into()), email: None };
    let answer = Envelope::new((Response { request_id: request.request_id }, person.clone(), ScopeChoices { scopes: scopes.build() }))?;

    let (response, signed_in, choices) = received(&answer).await?.get::<(Response, SignedInPerson, ScopeChoices)>()?;
    assert_eq!((response.request_id, signed_in), (asked, person));
    let read = rows_of(&choices.scopes)?
        .iter()
        .map(|row| Ok((row.entity_id(), row.get::<(Scope, Option<Registrable>)>()?)))
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
    assert_eq!(
        read,
        [
            (Some(entity(50)), (acme, Some(Registrable {}))),
            (Some(entity(51)), (globex, None)),
            (Some(entity(52)), (globex_east, Some(Registrable {}))),
        ]
    );
    Ok(())
}

#[tokio::test]
async fn a_registration_names_its_station_its_place_and_its_application_and_is_answered_with_its_grant() -> Outcome {
    let assignment = ScopeAssignment { station: Ref::new(station_record()) };
    let placement = Placement { scope: Ref::new(entity(51)), sub_scope: Some(Ref::new(entity(52))) };
    let asked = RequestId::new();
    let request =
        Envelope::new((Request { request_id: asked.clone() }, DoRegister {}, assignment.clone(), placement.clone(), for_application()))?;
    let (request, _, station_assigned, placed, first, application) =
        received(&request)
            .await?
            .get::<(Request, DoRegister, ScopeAssignment, Option<Placement>, Option<FirstScope>, Option<ForApplication>)>()?;
    assert_eq!((station_assigned, placed, first, application), (assignment, Some(placement), None, Some(for_application())));

    let answer = Envelope::new((Response { request_id: request.request_id }, for_application(), registration(40, REGISTERED)))?;
    let (response, application, kept) = received(&answer).await?.get::<(Response, ForApplication, Registration)>()?;
    assert_eq!((response.request_id, application, kept), (asked, for_application(), registration(40, REGISTERED)));
    Ok(())
}

// A person who is a member of no scope registers the station in a first scope that they name.
#[tokio::test]
async fn a_registration_in_a_first_scope_names_the_scope_in_place_of_a_placement() -> Outcome {
    let first = FirstScope { name: Name::new("Acme")? };
    let request = Envelope::new((
        Request { request_id: RequestId::new() },
        DoRegister {},
        ScopeAssignment { station: Ref::new(station_record()) },
        first.clone(),
        for_application(),
    ))?;
    let (_, _, _, placed, named, _) =
        received(&request)
            .await?
            .get::<(Request, DoRegister, ScopeAssignment, Option<Placement>, Option<FirstScope>, Option<ForApplication>)>()?;
    assert_eq!((placed, named), (None, Some(first)));
    Ok(())
}

// A station of Pylon 0.1.0 registers for no one application.
#[tokio::test]
async fn pylon_0_1_0s_registration_takes_the_same_components_without_an_application() -> Outcome {
    let request = Envelope::new((
        Request { request_id: RequestId::new() },
        DoRegister {},
        ScopeAssignment { station: Ref::new(station_record()) },
        Placement { scope: Ref::new(entity(50)), sub_scope: None },
    ))?;
    let (_, _, _, placed, _, application) =
        received(&request)
            .await?
            .get::<(Request, DoRegister, ScopeAssignment, Option<Placement>, Option<FirstScope>, Option<ForApplication>)>()?;
    assert_eq!((placed.map(|placement| placement.scope), application), (Some(Ref::new(entity(50))), None));
    Ok(())
}

#[tokio::test]
async fn an_end_of_a_registration_names_its_application_and_grant_and_is_answered_with_the_response_alone() -> Outcome {
    let asked = RequestId::new();
    let grant = ForGrant { grant: registration(40, REGISTERED).grant };
    let request = Envelope::new((Request { request_id: asked.clone() }, DoEndRegistration {}, for_application(), grant.clone()))?;
    let (request, _, application, ended) = received(&request).await?.get::<(Request, DoEndRegistration, ForApplication, ForGrant)>()?;
    assert_eq!((application, ended), (for_application(), grant));

    let answer = received(&Envelope::new(Response { request_id: request.request_id })?).await?;
    assert_eq!(answer.get::<Response>()?.request_id, asked);
    Ok(())
}

// An event's id is the hash of what it holds, so the registry imports the station's events as they
// were encoded, and the import's answer has nothing more to say.
#[tokio::test]
async fn an_import_hands_over_the_stations_events_unchanged_and_is_answered_with_the_response_alone() -> Outcome {
    let events = Events { events: b"the station's events, as Ankurah encodes them".to_vec() };
    let asked = RequestId::new();
    let request = Envelope::new((Request { request_id: asked.clone() }, DoImportEvents {}, events.clone()))?;
    let (request, _, imported) = received(&request).await?.get::<(Request, DoImportEvents, Events)>()?;
    assert_eq!(imported, events);

    let answer = received(&Envelope::new(Response { request_id: request.request_id })?).await?;
    assert_eq!(answer.get::<Response>()?.request_id, asked);
    Ok(())
}

// A station reads the fingerprints in the registry's order, since the first that names a driver
// decides.
#[tokio::test]
async fn a_catalog_is_answered_with_each_stock_then_each_fingerprint_in_the_registrys_order() -> Outcome {
    let asked = RequestId::new();
    let request = received(&Envelope::new((Request { request_id: asked.clone() }, DoListCatalog {}))?).await?;
    let (request, _) = request.get::<(Request, DoListCatalog)>()?;

    let formats = |format: &str| MimeTypes::new(vec![MimeType::new(format).unwrap()]).unwrap();
    let longer = PrinterFingerprint { driver: "LP 2844-Z".into(), document_formats: formats("application/x-zpl2") };
    let shorter = PrinterFingerprint { driver: "LP 2844".into(), document_formats: formats("application/x-epl2") };
    let mut entries = EntityRows::builder();
    entries.push(Some(entity(60)), stock("4x6", 101.6, Some(152.4)))?;
    entries.push(Some(entity(61)), stock("Continuous 2 inch", 50.8, None))?;
    entries.push(Some(entity(62)), longer.clone())?;
    entries.push(Some(entity(63)), shorter.clone())?;
    let answer = Envelope::new((Response { request_id: request.request_id }, Catalog { entries: entries.build() }))?;

    let (response, catalog) = received(&answer).await?.get::<(Response, Catalog)>()?;
    assert_eq!(response.request_id, asked);
    let rows = rows_of(&catalog.entries)?;
    let stocks = rows.iter().filter(|row| row.has::<Media>()).map(|row| row.get::<Media>()).collect::<Result<Vec<_>, _>>()?;
    let fingerprints = rows
        .iter()
        .filter(|row| row.has::<PrinterFingerprint>())
        .map(|row| row.get::<PrinterFingerprint>())
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(stocks, [stock("4x6", 101.6, Some(152.4)), stock("Continuous 2 inch", 50.8, None)]);
    assert_eq!(fingerprints, [longer, shorter]);
    Ok(())
}
