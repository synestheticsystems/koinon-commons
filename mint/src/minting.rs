//! Mints the definitions of every listed model without a node. An entity's operations are what
//! registering its model's declaration writes (`ankurah_commons::registration`). It keeps the
//! nonce and timestamp of a kept event of its file whose operations are exactly those, and
//! draws a fresh nonce and the current time otherwise. Its id follows from those, so minting a
//! kept mint again changes only what the declarations changed.

use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use ankurah::{
    core::schema::ModelStructDescriptor,
    proto::{Event, OperationSet},
    EntityId,
};
use ankurah_commons::{
    akev::{BuildError, Genesis},
    namespaces::Namespace,
    registration::{self, Definition},
};
use thiserror::Error;

use self::kept::Kept;
use crate::placement::{self, PlaceError, EXTENSION};

mod kept;

pub use kept::KeptError;

/// What minting made of one model: the file of its definitions, beside the struct that
/// declares it.
pub struct MintedFile {
    pub descriptor: &'static ModelStructDescriptor,
    /// The struct's source, which declares the model's id and each property's.
    pub source: PathBuf,
    /// The `.akev` file beside it.
    pub path: PathBuf,
    /// The id of the model's definition.
    pub model: EntityId,
    /// The id of each property's definition, by the field that declares the property, in the
    /// order the struct declares them.
    pub properties: Vec<(&'static str, EntityId)>,
    pub geneses: Vec<Genesis>,
    /// What the file needs that no kept event matched, whose nonce and timestamp were drawn.
    pub drawn: Vec<Definition>,
    /// The kept events of the file that nothing it needs matched.
    pub unneeded: Vec<EntityId>,
}

/// Why minting stopped.
#[derive(Debug, Error)]
pub enum MintError {
    #[error(transparent)]
    Placement(#[from] PlaceError),
    #[error("the kept mint: {0}")]
    Kept(#[from] KeptError),
    #[error("{label}: the operations do not build: {source}")]
    Build { label: &'static str, source: BuildError },
    #[error("the namespace {0} lists kinds of edge, which only the Ankurah Commons' own mint mints")]
    EdgeKinds(&'static str),
}

/// Mints every model of `namespaces`, keeping the nonces and timestamps of the kept mint under
/// `src`, the namespaces' parent directory.
pub fn mint(src: &Path, namespaces: &[Namespace]) -> Result<Vec<MintedFile>, MintError> {
    let kept = Kept::read(src)?;
    let mut files = Vec::new();
    for namespace in namespaces {
        if namespace.edge_kinds.is_some() {
            return Err(MintError::EdgeKinds(namespace.name));
        }
        let directory = src.join(namespace.name);
        for model in namespace.models {
            let descriptor = model();
            let source = placement::model_source(&directory, descriptor.name)?;
            let path = source.with_extension(EXTENSION);
            let mut keeping = Keeping::new(kept.events(&path));
            let built = |source| MintError::Build { label: descriptor.label, source };
            let geneses =
                registration::definitions(descriptor, |definition, operations| keeping.creation(definition, operations)).map_err(built)?;
            let id = |genesis: &Genesis| genesis.event().map(|event| event.entity_id).map_err(built);
            // The model's definition comes first, then each property's followed by its link's,
            // in the order the struct declares the properties.
            let model = id(&geneses[0])?;
            let properties = descriptor
                .properties
                .iter()
                .zip(geneses[1..].iter().step_by(2))
                .map(|(property, genesis)| Ok((property.field, id(genesis)?)))
                .collect::<Result<_, MintError>>()?;
            let (drawn, unneeded) = keeping.finish();
            files.push(MintedFile { descriptor, source, path, model, properties, geneses, drawn, unneeded });
        }
    }
    Ok(files)
}

/// The kept events of one file, and which of them the mint has kept.
struct Keeping<'k> {
    kept: &'k [Event],
    used: Vec<bool>,
    drawn: Vec<Definition>,
}

impl<'k> Keeping<'k> {
    fn new(kept: &'k [Event]) -> Self {
        Keeping { kept, used: vec![false; kept.len()], drawn: Vec::new() }
    }

    /// The nonce and timestamp of `definition`, whose operations are `operations`: those of the
    /// first kept event not yet used with exactly these operations, or drawn now.
    fn creation(&mut self, definition: Definition, operations: &OperationSet) -> ([u8; 32], u64) {
        match (0..self.kept.len()).find(|&index| !self.used[index] && self.kept[index].operations() == operations) {
            Some(index) => {
                self.used[index] = true;
                (*self.kept[index].nonce(), self.kept[index].timestamp())
            }
            None => {
                self.drawn.push(definition);
                drawn()
            }
        }
    }

    /// What was drawn, and the kept events nothing used.
    fn finish(self) -> (Vec<Definition>, Vec<EntityId>) {
        let unused = self.kept.iter().zip(&self.used).filter(|(_, used)| !**used).map(|(event, _)| event.entity_id).collect();
        (self.drawn, unused)
    }
}

/// The nonce and timestamp of an entity the kept mint does not hold: a fresh nonce, and now.
fn drawn() -> ([u8; 32], u64) {
    let since_1970 = SystemTime::now().duration_since(UNIX_EPOCH).expect("the clock reads after 1970");
    (rand::random(), since_1970.as_millis().try_into().expect("milliseconds since 1970 fit in 64 bits"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::copies::{copied, edit_and_restate};
    use ankurah_commons::akev;
    use koinon_commons::namespaces::NAMESPACES;
    use std::fs;

    /// The events of the file at `path`.
    fn events(path: &Path) -> Vec<Event> {
        akev::read(&[(path, &fs::read_to_string(path).unwrap())]).unwrap().remove(0)
    }

    /// Kinds of edge are records the Ankurah Commons' own mint mints, so a namespace that lists
    /// any is refused before anything is minted, and the refusal names it.
    #[test]
    fn a_namespace_that_lists_kinds_of_edge_is_refused_by_name() {
        use ankurah_commons::namespaces::MintedEdgeKinds;
        use koinon_commons::devices::Station;
        static WITH_EDGES: &[Namespace] = &[Namespace {
            name: "devices",
            models: &[<Station as ankurah::Model>::descriptor],
            edge_kinds: Some(MintedEdgeKinds { model: <Station as ankurah::Model>::descriptor, kinds: &[("serves", "a station serves")] }),
        }];
        let src = copied();
        let error = mint(src.path(), WITH_EDGES).err().expect("a namespace with kinds of edge is refused");
        assert!(matches!(error, MintError::EdgeKinds("devices")), "{error}");
    }

    /// A kept link that holds another optionality than its property declares: the mint draws
    /// the link anew, every other entity of the file keeps its nonce and timestamp, and the
    /// stale link is no longer needed.
    #[test]
    fn a_changed_need_draws_its_event_anew_and_leaves_the_kept_one_unneeded() {
        let src = copied();
        let device = src.path().join("devices").join("device.akev");
        let before = events(&device);
        edit_and_restate(&device, "optional: false", "optional: true");
        let kept = events(&device);
        let stale: Vec<EntityId> =
            kept.iter().map(|event| event.entity_id).filter(|id| before.iter().all(|event| event.entity_id != *id)).collect();
        assert_eq!(stale.len(), 1);

        let minted = mint(src.path(), NAMESPACES).unwrap();
        let file = minted.iter().find(|file| file.path == device).unwrap();
        assert_eq!(file.drawn, [Definition::Link("station")]);
        assert_eq!(file.unneeded, stale);
        // The model's definition, then `station`'s, then the link drawn anew.
        let drawn = &file.geneses[2];
        assert!(kept.iter().all(|event| *event.nonce() != drawn.nonce));
        for (index, (genesis, event)) in file.geneses.iter().zip(&before).enumerate().filter(|(index, _)| *index != 2) {
            assert_eq!((genesis.nonce, genesis.timestamp), (*event.nonce(), event.timestamp()), "the genesis at {index}");
        }
    }

    /// Each field carries the id of its property's definition, the genesis that makes its
    /// entity a property, in the order the struct declares the fields, and never the id of the
    /// link that gives the model the property.
    #[test]
    fn each_field_carries_the_id_of_the_definition_of_its_property() {
        use ankurah::proto::{ModelId, SystemModel};
        let src = copied();
        for file in mint(src.path(), NAMESPACES).unwrap() {
            let definitions = file
                .geneses
                .iter()
                .filter(|genesis| genesis.operations.memberships == [ModelId::System(SystemModel::Property)])
                .map(|genesis| genesis.event().unwrap().entity_id);
            let fields = file.descriptor.properties.iter().map(|property| property.field);
            assert_eq!(file.properties, fields.zip(definitions).collect::<Vec<_>>(), "{}", file.descriptor.label);
        }
    }
}
