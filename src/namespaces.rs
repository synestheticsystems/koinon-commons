//! The namespaces of the Koinon Commons, with the models each declares: what the mint tool
//! mints, and what a seeded node binds. The text of the kept mint is compiled in by `seed`
//! instead, so that a model can be listed, and minted, before it has a file. A test below
//! compares the list with the `#[derive(Model)]` structs in the namespaces' sources, so that a
//! model added there and not here fails.
//!
//! A namespace is listed as the Ankurah Commons lists its own, as its `Namespace`: the module
//! named `name`, kept in the directory of that name in the crate's `src/`, and the declarations
//! of its models. No Koinon namespace mints kinds of edge, so each has none.

use ankurah::Model;
use ankurah_commons::namespaces::Namespace;

use crate::{authority, catalog, devices, jobs, nodes, registry, replies};

/// Every namespace of the crate, and each one's models in the order its module declares them.
pub const NAMESPACES: &[Namespace] = &[
    Namespace {
        name: "authority",
        models: &[
            authority::ForApplication::descriptor,
            authority::Hosting::descriptor,
            authority::InvitationClaim::descriptor,
            authority::AllowsRequests::descriptor,
            authority::AtDevices::descriptor,
            authority::AtStations::descriptor,
            authority::Expiry::descriptor,
            authority::Retirement::descriptor,
            authority::SlipsVoided::descriptor,
            authority::Party::descriptor,
            authority::PermissionSlip::descriptor,
        ],
        edge_kinds: None,
    },
    Namespace {
        name: "catalog",
        models: &[catalog::Catalog::descriptor, catalog::PrinterFingerprint::descriptor, catalog::operations::DoListCatalog::descriptor],
        edge_kinds: None,
    },
    Namespace {
        name: "devices",
        models: &[
            devices::Device::descriptor,
            devices::DeviceListing::descriptor,
            devices::Media::descriptor,
            devices::Printer::descriptor,
            devices::Scale::descriptor,
            devices::ScaleReading::descriptor,
            devices::Station::descriptor,
            devices::operations::DoListDevices::descriptor,
        ],
        edge_kinds: None,
    },
    Namespace {
        name: "jobs",
        models: &[
            jobs::JobAttempt::descriptor,
            jobs::JobFailure::descriptor,
            jobs::ForDevice::descriptor,
            jobs::JobLabel::descriptor,
            jobs::PrintJob::descriptor,
            jobs::operations::DoPrint::descriptor,
            jobs::operations::DoWeigh::descriptor,
        ],
        edge_kinds: None,
    },
    Namespace {
        name: "nodes",
        models: &[nodes::Events::descriptor, nodes::NodeLink::descriptor, nodes::operations::DoImportEvents::descriptor],
        edge_kinds: None,
    },
    Namespace {
        name: "registry",
        models: &[
            registry::ForGrant::descriptor,
            registry::IdTokenSignIn::descriptor,
            registry::KeptRegistrations::descriptor,
            registry::NodeGrant::descriptor,
            registry::Registration::descriptor,
            registry::FirstScope::descriptor,
            registry::Placement::descriptor,
            registry::Registrable::descriptor,
            registry::Scope::descriptor,
            registry::ScopeAssignment::descriptor,
            registry::ScopeChoices::descriptor,
            registry::SignedInPerson::descriptor,
            registry::operations::DoEndRegistration::descriptor,
            registry::operations::DoListRegistrations::descriptor,
            registry::operations::DoRegister::descriptor,
        ],
        edge_kinds: None,
    },
    Namespace { name: "replies", models: &[replies::NotUnderstood::descriptor, replies::Refusal::descriptor], edge_kinds: None },
];

/// The guard that keeps `NAMESPACES` complete, so that a model added to a namespace, or a new
/// namespace, cannot be forgotten in the list, and that keeps every model a component of the
/// envelope as well as one that is stored. It relies on one convention: a model is a struct with
/// `#[derive(Model, ..)]` written directly on it, in a file under its namespace's directory in
/// `src/`. A model declared any other way is not seen, and neither is one in a file named
/// `tests.rs`, where a test declares models of its own that belong to no namespace.
#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        collections::{BTreeMap, BTreeSet},
        fs,
        path::Path,
    };
    use syn::{punctuated::Punctuated, Token};

    /// Model names by namespace.
    type Models = BTreeMap<String, BTreeSet<String>>;

    /// Each model the sources under `src` declare, by the directory of `src` it is declared under,
    /// with whether its struct derives `EnvelopeComponent` too. A directory that declares no model
    /// is no namespace.
    fn declared(src: &Path) -> BTreeMap<String, BTreeMap<String, bool>> {
        let mut declared = BTreeMap::new();
        for entry in fs::read_dir(src).unwrap() {
            let path = entry.unwrap().path();
            let mut models = BTreeMap::new();
            if path.is_dir() {
                collect(&path, &mut models);
            }
            if !models.is_empty() {
                declared.insert(path.file_name().unwrap().to_str().unwrap().to_owned(), models);
            }
        }
        declared
    }

    fn sources() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
    }

    /// Adds to `models` every struct that a Rust file under `dir`, other than a test's, derives
    /// `Model` for, with whether it derives `EnvelopeComponent` too.
    fn collect(dir: &Path, models: &mut BTreeMap<String, bool>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect(&path, models);
            } else if path.extension().is_some_and(|extension| extension == "rs") && !path.ends_with("tests.rs") {
                let file = syn::parse_file(&fs::read_to_string(&path).unwrap()).unwrap();
                for item in file.items {
                    if let syn::Item::Struct(item) = item {
                        if derives(&item.attrs, "Model") {
                            models.insert(item.ident.to_string(), derives(&item.attrs, "EnvelopeComponent"));
                        }
                    }
                }
            }
        }
    }

    fn derives(attributes: &[syn::Attribute], name: &str) -> bool {
        attributes.iter().filter(|attribute| attribute.path().is_ident("derive")).any(|attribute| {
            let derived = attribute.parse_args_with(Punctuated::<syn::Path, Token![,]>::parse_terminated).unwrap();
            derived.iter().any(|path| path.segments.last().is_some_and(|segment| segment.ident == name))
        })
    }

    #[test]
    fn the_list_names_every_model_the_namespaces_declare_and_each_label_once() {
        let listed: Vec<(&str, &str, &str)> = NAMESPACES
            .iter()
            .flat_map(|namespace| namespace.models.iter().map(|model| (namespace.name, model().name, model().label)))
            .collect();
        let mut models = Models::new();
        for &(namespace, model, _) in &listed {
            assert!(models.entry(namespace.to_owned()).or_default().insert(model.to_owned()), "{model} is listed twice");
        }
        let declared: Models =
            declared(&sources()).into_iter().map(|(namespace, models)| (namespace, models.into_keys().collect())).collect();
        assert_eq!(models, declared);
        let labels: BTreeSet<&str> = listed.iter().map(|&(_, _, label)| label).collect();
        assert_eq!(labels.len(), listed.len(), "a label is listed twice");
    }

    // So that no model is one that is stored and not sent.
    #[test]
    fn every_model_the_namespaces_declare_derives_envelope_component() {
        let unsent: Vec<String> = declared(&sources()).into_values().flatten().filter(|&(_, sent)| !sent).map(|(model, _)| model).collect();
        assert!(unsent.is_empty(), "{unsent:?} derive Model and not EnvelopeComponent");
    }
}
