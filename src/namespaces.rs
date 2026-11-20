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

use crate::{devices, nodes};

/// Every namespace of the crate, and each one's models in the order its module declares them.
pub const NAMESPACES: &[Namespace] = &[
    Namespace {
        name: "devices",
        models: &[
            devices::Device::descriptor,
            devices::Media::descriptor,
            devices::Printer::descriptor,
            devices::Scale::descriptor,
            devices::Station::descriptor,
        ],
        edge_kinds: None,
    },
    Namespace { name: "nodes", models: &[nodes::NodeLink::descriptor], edge_kinds: None },
];

/// The guard that keeps `NAMESPACES` complete, so that a model added to a namespace, or a new
/// namespace, cannot be forgotten in the list. It relies on one convention: a model is a struct
/// with `#[derive(Model, ..)]` written directly on it, in a file under its namespace's directory
/// in `src/`. A model declared any other way is not seen.
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

    /// The models the sources under `src` declare, by the directory of `src` they are declared
    /// under. A directory that declares no model is no namespace.
    fn declared(src: &Path) -> Models {
        let mut models = Models::new();
        for entry in fs::read_dir(src).unwrap() {
            let path = entry.unwrap().path();
            let mut names = BTreeSet::new();
            if path.is_dir() {
                collect(&path, &mut names);
            }
            if !names.is_empty() {
                models.insert(path.file_name().unwrap().to_str().unwrap().to_owned(), names);
            }
        }
        models
    }

    /// Adds to `names` every struct that a Rust file under `dir` derives `Model` for.
    fn collect(dir: &Path, names: &mut BTreeSet<String>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect(&path, names);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                let file = syn::parse_file(&fs::read_to_string(&path).unwrap()).unwrap();
                for item in file.items {
                    if let syn::Item::Struct(item) = item {
                        if derives_model(&item.attrs) {
                            names.insert(item.ident.to_string());
                        }
                    }
                }
            }
        }
    }

    fn derives_model(attributes: &[syn::Attribute]) -> bool {
        attributes.iter().filter(|attribute| attribute.path().is_ident("derive")).any(|attribute| {
            let derived = attribute.parse_args_with(Punctuated::<syn::Path, Token![,]>::parse_terminated).unwrap();
            derived.iter().any(|path| path.segments.last().is_some_and(|segment| segment.ident == "Model"))
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
        assert_eq!(models, declared(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src")));
        let labels: BTreeSet<&str> = listed.iter().map(|&(_, _, label)| label).collect();
        assert_eq!(labels.len(), listed.len(), "a label is listed twice");
    }
}
