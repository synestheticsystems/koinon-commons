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
    "devices/device.akev",
    "devices/media.akev",
    "devices/printer.akev",
    "devices/scale.akev",
    "devices/station.akev",
    "nodes/node_link.akev",
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
        devices::*,
        mime_type::{MimeType, MimeTypes},
        namespaces::NAMESPACES,
        nodes::*,
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
    use ankurah_commons::dimension::Dimension;
    use ankurah_storage_sled::SledStorageEngine;
    use ankurah_storage_sqlite::SqliteStorageEngine;
    use serde::Serialize;
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

    /// What a component holds, so that a component as created and as read compare whole.
    fn held<T: Serialize>(component: &T) -> serde_json::Value {
        serde_json::to_value(component).unwrap()
    }

    /// What the component `view` reads holds.
    fn read<V: View>(view: &V) -> serde_json::Value
    where
        V::Model: Serialize,
    {
        held(&view.to_model().unwrap())
    }

    /// One record of each namespace, as a station makes them: its own record, a printer with
    /// the stock it is loaded with, a scale, and its link to a server it signed in to. Each
    /// reads back, fetched by its components, as it was created.
    #[tokio::test]
    async fn a_seeded_node_creates_and_fetches_a_record_of_each_namespace() {
        creates_and_fetches_a_record_of_each_namespace(sled().await).await;
        creates_and_fetches_a_record_of_each_namespace(sqlite().await).await;
    }

    async fn creates_and_fetches_a_record_of_each_namespace<SE: StorageEngine + Send + Sync + 'static>(engine: SE) {
        let node = seeded(engine).await;
        let context = node.context_async(DEFAULT_CONTEXT).await.unwrap();
        let station = Station { public_key: "station key".to_owned(), name: "Pack bench 3".to_owned() };
        let link = NodeLink { public_key: "server key".to_owned(), address: "https://pylon.example".to_owned(), name: "Server".to_owned() };
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
        assert_eq!((station_view.id(), read(&station_view)), (station_id, held(&station)));
        assert_eq!(read(&fetched::<NodeLinkView>(&context).await), held(&link));
        let (device_view, printer_view, media_view) = fetched::<(DeviceView, PrinterView, MediaView)>(&context).await;
        assert_eq!([read(&device_view), read(&printer_view), read(&media_view)], [held(&printer_device), held(&printer), held(&media)]);
        let (device_view, scale_view) = fetched::<(DeviceView, ScaleView)>(&context).await;
        assert_eq!([read(&device_view), read(&scale_view)], [held(&scale_device), held(&Scale {})]);
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
