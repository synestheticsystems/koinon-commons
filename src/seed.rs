//! Seeding a node: the kept mint's events, read from the `.akev` text compiled into this crate,
//! imported into a durable node's store, so that the Koinon Commons' models bind to the minted
//! definitions there rather than minting their own.

use std::path::Path;

use ankurah::{
    policy::PolicyAgent,
    proto::{Attested, Event},
    storage::StorageEngine,
    Node,
};
use ankurah_commons::akev;
use thiserror::Error;

/// Why the Koinon Commons' kept mint was not seeded: the reason the Ankurah Commons gives for a
/// kept mint, named as this crate's, so that a node's start-up, which seeds several crates in a
/// row, says which one failed.
#[derive(Debug, Error)]
#[error("the Koinon Commons: {0}")]
pub struct SeedError(pub ankurah_commons::SeedError);

/// Imports into `node` the kept mint of the Koinon Commons: the definitions of its models. A
/// model of the Koinon Commons then binds to its minted definitions, and registering it mints
/// nothing, so that a record one node makes of it reads as the same record on any other.
///
/// `node` must be durable, with its system root created or loaded. Seed it with the Ankurah
/// Commons, then with this: the entities of these models also carry its components. Seed it
/// before it first binds any of these models, which a fetch, a query, a get or a create does as
/// well as registration: a model declares its minted id, so binding it on a node not yet seeded
/// fails with `RegistrationError::ExplicitModelIdNotFound`. Seeding a node that holds the kept
/// mint already changes nothing, so a node can be seeded at every start.
pub async fn seed<SE, PA>(node: &Node<SE, PA>) -> Result<(), SeedError>
where
    SE: StorageEngine + Send + Sync + 'static,
    PA: PolicyAgent + Send + Sync + 'static,
{
    node.import_events(&kept_mint()?).await.map_err(|error| SeedError(ankurah_commons::SeedError::Import(Box::new(error))))
}

/// Each file's path under `src/` and its text, compiled in.
macro_rules! kept {
    ($($path:literal),* $(,)?) => {
        &[$(($path, include_str!($path))),*]
    };
}

/// The kept mint, as compiled into the crate: each model's `.akev` file, by its path under
/// `src/`.
const KEPT_MINT: &[(&str, &str)] = kept![
    "authority/applications/for_application.akev",
    "authority/applications/hosting.akev",
    "authority/applications/invitation_claim.akev",
    "authority/requests/claim_invitation.akev",
    "authority/requests/retire.akev",
    "authority/requests/void_slips.akev",
    "authority/restrictions/allows_requests.akev",
    "authority/restrictions/at_devices.akev",
    "authority/restrictions/at_stations.akev",
    "authority/restrictions/expiry.akev",
    "authority/revocation/retirement.akev",
    "authority/revocation/slips_voided.akev",
    "authority/signatures/presented_slip.akev",
    "authority/signatures/signed_hosting.akev",
    "authority/signatures/signed_request.akev",
    "authority/signatures/signed_retirement.akev",
    "authority/slips/party.akev",
    "authority/slips/permission_slip.akev",
    "catalog/printer_fingerprint.akev",
    "catalog/requests/get_catalog.akev",
    "devices/device.akev",
    "devices/printers/media.akev",
    "devices/printers/printer.akev",
    "devices/requests/list_devices.akev",
    "devices/scales/scale.akev",
    "devices/scales/scale_reading.akev",
    "devices/station.akev",
    "jobs/attempts/job_attempt.akev",
    "jobs/attempts/job_failure.akev",
    "jobs/for_device.akev",
    "jobs/job_id.akev",
    "jobs/job_label.akev",
    "jobs/printing/document_follows.akev",
    "jobs/printing/print_job.akev",
    "jobs/requests/print.akev",
    "jobs/requests/weigh.akev",
    "nodes/events.akev",
    "nodes/node_link.akev",
    "nodes/requests/push_events.akev",
    "registry/for_grant.akev",
    "registry/id_token_sign_in.akev",
    "registry/node_grant.akev",
    "registry/registration.akev",
    "registry/requests/hello.akev",
    "registry/requests/register.akev",
    "registry/requests/sign_in.akev",
    "registry/requests/stop_sharing.akev",
    "registry/scopes/first_scope.akev",
    "registry/scopes/placement.akev",
    "registry/scopes/registrable.akev",
    "registry/scopes/scope.akev",
    "registry/scopes/scope_assignment.akev",
    "registry/signed_in_person.akev",
    "replies/done.akev",
    "replies/not_understood.akev",
    "replies/refusal.akev",
];

/// The events of the kept mint.
fn kept_mint() -> Result<Vec<Attested<Event>>, SeedError> {
    let files: Vec<(&Path, &str)> = KEPT_MINT.iter().map(|&(path, text)| (Path::new(path), text)).collect();
    let events = akev::read(&files).map_err(|error| SeedError(ankurah_commons::SeedError::Unreadable(error)))?;
    Ok(events.into_iter().flatten().map(Attested::from).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        authority::{AllowsRequests, AtStations, Expiry, ExpiryView, Party, PermissionSlip, PresentedSlip, PresentedSlipView, Signed},
        codec::{EnvelopeBuilder, Minted, SealedEnvelope},
        component_ids::ComponentIds,
        devices::*,
        jobs::{requests::Print, *},
        mime_type::{MimeType, MimeTypes},
        name::Name,
        namespaces::NAMESPACES,
        nodes::*,
        public_key::{PublicKey, PublicKeys},
    };
    use ankurah::{
        core::schema::registration::RegistrationError,
        entity::LocalTrxEntity,
        model::{ComponentViews, Mutable},
        policy::DEFAULT_CONTEXT,
        property::Ref,
        proto::EntityState,
        Context, EntityId, Model, ModelId, PermissiveAgent, PropertyId, View,
    };
    use ankurah_commons::{dimension::Dimension, property::Timestamp};
    use ankurah_storage_sled::SledStorageEngine;
    use ankurah_storage_sqlite::SqliteStorageEngine;
    use ed25519_dalek::SigningKey;
    use std::sync::Arc;

    /// The storage engines a node of these models runs on: Sled, and SQLite, which a station
    /// keeps its journal in. Each test runs on both, as the Ankurah Commons' own do.
    async fn sled() -> SledStorageEngine {
        SledStorageEngine::new_test().unwrap()
    }

    async fn sqlite() -> SqliteStorageEngine {
        SqliteStorageEngine::open_in_memory().await.unwrap()
    }

    /// A durable node with a system of its own, seeded as a node that uses these models seeds
    /// itself.
    async fn seeded<SE: StorageEngine + Send + Sync + 'static>(engine: SE) -> Node<SE, PermissiveAgent> {
        let node = Node::new_durable(Arc::new(engine), PermissiveAgent::new());
        node.system.create().await.unwrap();
        ankurah_commons::seed(&node).await.unwrap();
        seed(&node).await.unwrap();
        node.wait_ready().await.unwrap();
        node
    }

    fn key(seed: u8) -> PublicKey {
        PublicKey::from(SigningKey::from_bytes(&[seed; 32]).verifying_key())
    }

    /// Registration binds a model through the catalog's local resolution before it would
    /// register anything; every model resolves there to the minted definition it declares, and
    /// each of its properties to the id its field declares, so registering it mints nothing and
    /// nothing binds by name.
    #[tokio::test]
    async fn seeding_a_fresh_node_binds_every_model_to_its_minted_definition() {
        every_model_binds_to_its_minted_definition(sled().await).await;
        every_model_binds_to_its_minted_definition(sqlite().await).await;
    }

    async fn every_model_binds_to_its_minted_definition<SE: StorageEngine + Send + Sync + 'static>(engine: SE) {
        let node = seeded(engine).await;
        let epoch = node.system.system_epoch().unwrap();
        for descriptor in NAMESPACES.iter().flat_map(|namespace| namespace.models).map(|model| model()) {
            let declared = EntityId::from_base64(descriptor.explicit_id.expect("every model declares its minted id")).unwrap();
            assert_eq!(descriptor.bind_local(&node.catalog, epoch).unwrap(), ModelId::EntityId(declared), "{}", descriptor.label);
            for property in descriptor.properties {
                let declared = EntityId::from_base64(property.explicit_id.expect("every property declares its minted id")).unwrap();
                assert_eq!(property.resolved_id(epoch).unwrap(), PropertyId::EntityId(declared), "{}.{}", descriptor.label, property.field);
            }
        }
    }

    /// Stages `component` on `entity`, an entity its transaction creates, as one more of its
    /// components, before its genesis is made. Ankurah has no typed way yet to create an entity
    /// with several components in one step.
    async fn stage<M: Model>(context: &Context, entity: &LocalTrxEntity, component: &M) {
        let model = context.resolve_model_id::<M>().await.unwrap();
        component.initialize_new_entity(entity, model, entity.system_epoch()).unwrap();
    }

    /// The one entity the node holds with every component of `R`.
    async fn fetched<R: ComponentViews>(context: &Context) -> R {
        let mut matched: Vec<R> = context.fetch("true").await.unwrap();
        assert_eq!(matched.len(), 1, "{} entities match", matched.len());
        matched.remove(0)
    }

    /// The component `view` reads, so that a component as created and as read compare whole.
    fn read<V: View>(view: &V) -> V::Model {
        view.to_model().unwrap()
    }

    /// Each kind of record a station makes: its own record, a printer with the stock it is
    /// loaded with, a scale, and its link to a server it signed in to. Each reads back, fetched
    /// by its components, as it was created.
    #[tokio::test]
    async fn a_seeded_node_creates_and_fetches_each_kind_of_record_a_station_makes() {
        creates_and_fetches_each_kind_of_record_a_station_makes(sled().await).await;
        creates_and_fetches_each_kind_of_record_a_station_makes(sqlite().await).await;
    }

    async fn creates_and_fetches_each_kind_of_record_a_station_makes<SE: StorageEngine + Send + Sync + 'static>(engine: SE) {
        let node = seeded(engine).await;
        let context = node.context_async(DEFAULT_CONTEXT).await.unwrap();
        let station = Station { public_key: key(2), name: "Pack bench 3".to_owned() };
        let link = NodeLink { public_key: key(4), address: "https://pylon.example".to_owned(), name: "Server".to_owned() };
        let transaction = context.begin();
        let station_id = transaction.create(&station).await.unwrap().id();
        transaction.create(&link).await.unwrap();
        transaction.commit().await.unwrap();

        let device = |system_name: &str, name: &str| Device {
            station: Ref::new(station_id),
            system_name: system_name.to_owned(),
            name: name.to_owned(),
        };
        let (printer_device, scale_device) = (device("Zebra_GK420d", "Pack bench 3 labels"), device("SN 4411", "Pack bench 3 scale"));
        let printer = Printer {
            driver: "Zebra ZPL Label Printer".to_owned(),
            document_formats: MimeTypes::new(vec![
                MimeType::new("application/vnd.cups-raw").unwrap(),
                MimeType::new("application/pdf").unwrap(),
            ])
            .unwrap(),
        };
        let media = Media {
            name: "4x6".to_owned(),
            width: Dimension::from_millimeters(101.6).unwrap(),
            height: Some(Dimension::from_millimeters(152.4).unwrap()),
        };
        let transaction = context.begin();
        let created = transaction.create(&printer_device).await.unwrap();
        stage(&context, created.entity(), &printer).await;
        stage(&context, created.entity(), &media).await;
        let created = transaction.create(&scale_device).await.unwrap();
        stage(&context, created.entity(), &Scale {}).await;
        transaction.commit().await.unwrap();

        let station_view: StationView = fetched(&context).await;
        assert_eq!((station_view.id(), read(&station_view)), (station_id, station));
        assert_eq!(read(&fetched::<NodeLinkView>(&context).await), link);
        let (device_view, printer_view, media_view) = fetched::<(DeviceView, PrinterView, MediaView)>(&context).await;
        assert_eq!((read(&device_view), read(&printer_view), read(&media_view)), (printer_device, printer, media));
        let (device_view, scale_view) = fetched::<(DeviceView, ScaleView)>(&context).await;
        assert_eq!((read(&device_view), read(&scale_view)), (scale_device, Scale {}));
    }

    /// What a station keeps of a print it took, the components its request and its answer carried,
    /// and what a registry keeps of a slip it issued, the carrier as the application's key signed
    /// it with the slip's expiry beside it: each stored as a message carried it, and fetched back
    /// whole, so that a record keeps a component as it travelled.
    #[tokio::test]
    async fn a_seeded_node_stores_a_stations_job_and_a_carried_slip_as_messages_carried_them() {
        stores_a_stations_job_and_a_carried_slip(sled().await).await;
        stores_a_stations_job_and_a_carried_slip(sqlite().await).await;
    }

    async fn stores_a_stations_job_and_a_carried_slip<SE: StorageEngine + Send + Sync + 'static>(engine: SE) {
        let node = seeded(engine).await;
        let context = node.context_async(DEFAULT_CONTEXT).await.unwrap();
        let device = ForDevice { device: Ref::new(EntityId::from_bytes([11; 32])) };
        let job = JobId { job: "order-1001-label".to_owned() };
        let label = JobLabel { name: "Order 1001".to_owned(), asked_by: None };
        let print = PrintJob {
            document_format: MimeType::new("application/x-zpl2").unwrap(),
            copies: Copies::new(2).unwrap(),
            document_hash: DocumentHash::of(b"^XA^FDOrder 1001^FS^XZ"),
        };
        let attempt = JobAttempt {
            state: AttemptState::Failed,
            accepted: Timestamp::from_unix_millis(1_794_600_000_200),
            finished: Some(Timestamp::from_unix_millis(1_794_600_001_200)),
        };
        let failure = JobFailure { code: JobFailure::DEVICE_TIMEOUT.to_owned(), message: "the printer did not answer".to_owned() };
        let transaction = context.begin();
        let created = transaction.create(&device).await.unwrap();
        stage(&context, created.entity(), &job).await;
        stage(&context, created.entity(), &label).await;
        stage(&context, created.entity(), &print).await;
        stage(&context, created.entity(), &attempt).await;
        stage(&context, created.entity(), &failure).await;
        transaction.commit().await.unwrap();

        let expiry = Expiry { expires: Timestamp::from_unix_millis(1_795_204_800_000) };
        let mut envelope = EnvelopeBuilder::new();
        envelope
            .add_must_understand(&PermissionSlip {
                issued: Timestamp::from_unix_millis(1_794_600_000_000),
                slip_id: EntityId::from_bytes([30; 32]),
            })
            .unwrap()
            .add_must_understand(&Party { key: key(3), name: Name::new("Warehouse label PC").unwrap() })
            .unwrap()
            .add_must_understand(&AtStations { stations: PublicKeys::new(vec![key(2)]).unwrap() })
            .unwrap()
            .add_must_understand(&AllowsRequests { requests: ComponentIds::new(vec![Print::ID]).unwrap() })
            .unwrap()
            .add_must_understand(&expiry)
            .unwrap();
        let slip = PresentedSlip::sign(SealedEnvelope::new(envelope.encode()), &SigningKey::from_bytes(&[1; 32]));
        let transaction = context.begin();
        let created = transaction.create(&slip).await.unwrap();
        stage(&context, created.entity(), &expiry).await;
        transaction.commit().await.unwrap();

        let (device_view, job_view, label_view, print_view, attempt_view, failure_view) =
            fetched::<(ForDeviceView, JobIdView, JobLabelView, PrintJobView, JobAttemptView, JobFailureView)>(&context).await;
        assert_eq!(
            (read(&device_view), read(&job_view), read(&label_view), read(&print_view), read(&attempt_view), read(&failure_view)),
            (device, job, label, print, attempt, failure)
        );
        let (slip_view, expiry_view) = fetched::<(PresentedSlipView, ExpiryView)>(&context).await;
        let stored = read(&slip_view);
        assert_eq!((&stored, read(&expiry_view)), (&slip, expiry));
        assert_eq!(stored.verify(), Ok(()), "the stored slip's signature verifies over the bytes it keeps");
    }

    /// Every event and state the store holds for each entity of the kept mint, as stored.
    async fn stored<SE: StorageEngine + Send + Sync + 'static>(
        node: &Node<SE, PermissiveAgent>,
    ) -> Vec<(Vec<Attested<Event>>, Attested<EntityState>)> {
        let mut stored = Vec::new();
        for event in kept_mint().unwrap() {
            let id = event.payload.entity_id;
            stored.push((node.storage.dump_entity_events(id).await.unwrap(), node.storage.get_state(id).await.unwrap()));
        }
        stored
    }

    #[tokio::test]
    async fn seeding_a_seeded_node_again_changes_no_stored_event_or_state() {
        seeding_again_changes_nothing(sled().await).await;
        seeding_again_changes_nothing(sqlite().await).await;
    }

    async fn seeding_again_changes_nothing<SE: StorageEngine + Send + Sync + 'static>(engine: SE) {
        let node = seeded(engine).await;
        let before = stored(&node).await;
        seed(&node).await.unwrap();
        assert_eq!(stored(&node).await, before);
    }

    /// Each model declares its minted id, so on a node that was not seeded it fails to
    /// register, rather than minting definitions of its own that no other node would share.
    #[tokio::test]
    async fn without_the_seed_a_model_fails_to_register_with_a_typed_error() {
        let node = Node::new_durable(Arc::new(sled().await), PermissiveAgent::new());
        node.system.create().await.unwrap();
        let context = node.context_async(DEFAULT_CONTEXT).await.unwrap();
        let declared = EntityId::from_base64(Device::descriptor().explicit_id.unwrap()).unwrap();
        let error = context.resolve_model_id::<Device>().await.unwrap_err();
        assert!(matches!(error, RegistrationError::ExplicitModelIdNotFound { model } if model == declared), "{error}");
        assert_eq!(node.catalog.model_labels(), []);
    }
}
