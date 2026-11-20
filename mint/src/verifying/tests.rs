use super::*;
use crate::copies::{copied, edit_and_restate, files, mint, sources};
use ankurah_commons::akev::{self, Problem};
use koinon_commons::namespaces::NAMESPACES;
use std::fs;
use tempfile::TempDir;

/// A copy of the sources with the kept mint minted again.
fn minted() -> TempDir {
    let src = copied();
    mint(src.path());
    src
}

fn failure(src: &TempDir) -> VerifyError {
    verify(src.path(), NAMESPACES).unwrap_err()
}

/// Each entity keeps its nonce and timestamp, so minting the kept mint again under the same
/// declarations and the same encoding of Ankurah's events writes every file as it was, the ids
/// the declarations state with them.
#[test]
fn minting_the_kept_mint_again_changes_no_file_and_it_verifies() {
    let src = minted();
    assert_eq!(files(src.path()), files(&sources()));
    let verified = verify(src.path(), NAMESPACES).unwrap();
    let models = NAMESPACES.iter().map(|namespace| namespace.models.len()).sum::<usize>();
    let definitions: usize = NAMESPACES.iter().flat_map(|namespace| namespace.models).map(|model| 1 + 2 * model().properties.len()).sum();
    assert_eq!((verified.files, verified.events), (models, definitions));
}

/// `devices.station` as its struct declares it, but for the id.
#[derive(ankurah::Model, Debug, Clone)]
#[model(label = "devices.station")]
#[model(id = "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE")]
#[model(no_ffi)]
pub struct Station {
    #[active_type(LWW)]
    pub public_key: String,
    #[active_type(LWW)]
    pub name: String,
}

/// After a mint that changed a model's id, its struct declares the old id until the tool is
/// built again; the file's id is the one that holds.
#[test]
fn a_struct_that_declares_another_id_than_its_file_states_fails_naming_the_file() {
    static WITH_ANOTHER_ID: &[Namespace] =
        &[Namespace { name: "devices", models: &[<Station as ankurah::Model>::descriptor], edge_kinds: None }];
    let src = tempfile::tempdir().unwrap();
    let devices = src.path().join("devices");
    fs::create_dir(&devices).unwrap();
    for file in ["station.rs", "station.akev"] {
        fs::copy(sources().join("devices").join(file), devices.join(file)).unwrap();
    }
    let error = verify(src.path(), WITH_ANOTHER_ID).unwrap_err();
    assert_eq!(error.path, devices.join("station.akev"));
    let kept = EntityId::from_base64(<koinon_commons::devices::Station as ankurah::Model>::descriptor().explicit_id.unwrap()).unwrap();
    assert!(
        matches!(error.failure, Failure::OtherId { stated, declared: Some("AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE") } if stated == kept),
        "{error}"
    );
}

/// A nonce changed by a character reads as another event than its header names.
#[test]
fn a_changed_nonce_fails_the_read_naming_the_file_and_the_line() {
    let src = minted();
    let file = src.path().join("devices").join("station.akev");
    let text = fs::read_to_string(&file).unwrap();
    let digit = text.find("nonce: ").unwrap() + "nonce: ".len();
    let changed = if &text[digit..digit + 1] == "A" { "B" } else { "A" };
    fs::write(&file, format!("{}{changed}{}", &text[..digit], &text[digit + 1..])).unwrap();
    let error = failure(&src);
    assert_eq!(error.path, file);
    let read = match error.failure {
        Failure::Unread(read) => read,
        failure => panic!("{failure}"),
    };
    assert!(matches!(*read, ReadError { line: 1, problem: Problem::NotTheHeadersEntity { .. }, .. }), "{read}");
}

/// A file must state what the sources declare, so that a mint left behind by a change to a
/// struct fails, though every file reads.
#[test]
fn a_file_that_states_other_events_than_the_sources_need_fails_naming_it() {
    let src = minted();
    let device = src.path().join("devices").join("device.akev");
    edit_and_restate(&device, "optional: false", "optional: true");
    let error = failure(&src);
    assert_eq!(error.path, device);
    assert!(matches!(error.failure, Failure::NotAsDeclared(Definition::Link("station"))), "{error}");
}

/// A genesis that no declaration needs, beside every one that the declarations do.
#[test]
fn a_kept_event_no_declaration_needs_fails_naming_its_file_and_id() {
    let src = minted();
    let device = src.path().join("devices").join("device.akev");
    let spare = format!(
        "member: system:model\n\n[{}]\nnonce: {}, timestamp: 1970-01-01T00:00:00.000Z\nlww: {{ name: \"spare\" }}\n",
        EntityId::from_bytes([0; 32]),
        EntityId::from_bytes([1; 32])
    );
    edit_and_restate(&device, "member: system:model\n", &spare);
    let text = fs::read_to_string(&device).unwrap();
    let spare = akev::read(&[(&device, &text)]).unwrap().remove(0)[1].entity_id;
    let error = failure(&src);
    assert_eq!(error.path, device);
    assert!(matches!(error.failure, Failure::Unneeded(id) if id == spare), "{error}");
}

#[test]
fn a_missing_or_stray_file_fails_naming_it() {
    let src = minted();
    let device = src.path().join("devices").join("device.akev");
    let kept = fs::read_to_string(&device).unwrap();
    fs::remove_file(&device).unwrap();
    let error = failure(&src);
    assert_eq!(error.path, device);
    assert!(matches!(error.failure, Failure::Missing), "{error}");
    fs::write(&device, kept).unwrap();

    let stray = src.path().join("devices").join("tray.akev");
    fs::write(&stray, "").unwrap();
    let error = failure(&src);
    assert_eq!(error.path, stray);
    assert!(matches!(error.failure, Failure::Stray), "{error}");
}

/// Kinds of edge are minted by the Ankurah Commons' own mint alone, so verifying a namespace
/// that lists any fails at that namespace's directory, before any file is read.
#[test]
fn a_namespace_that_lists_kinds_of_edge_fails_at_its_directory() {
    use ankurah_commons::namespaces::MintedEdgeKinds;
    static WITH_EDGES: &[Namespace] = &[Namespace {
        name: "devices",
        models: &[<koinon_commons::devices::Station as ankurah::Model>::descriptor],
        edge_kinds: Some(MintedEdgeKinds {
            model: <koinon_commons::devices::Station as ankurah::Model>::descriptor,
            kinds: &[("serves", "a station serves")],
        }),
    }];
    let src = minted();
    let error = verify(src.path(), WITH_EDGES).unwrap_err();
    assert_eq!(error.path, src.path().join("devices"));
    assert!(matches!(&error.failure, Failure::Unminted(failure) if matches!(**failure, MintError::EdgeKinds("devices"))), "{error}");
}
