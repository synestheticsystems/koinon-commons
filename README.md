# Koinon Commons

Definitions of the Ankurah models that Koinon programs share, so that two programs, or two
nodes, mean the same thing by "a device" or "a station". A layer on the
[Ankurah Commons](https://github.com/ankurah/commons), in its shape.

**Status: draft, with a mint kept, and no id final.** The definitions of every model in the
two namespaces, 6 models with 13 properties and their links, are minted, unsigned, in the
`.akev` files beside the structs, in the format of the Ankurah revision this crate builds
against.

## Layout

One crate, `koinon-commons`, with a module per namespace, one file per model, and beside each
model's file the `.akev` file of its minted definitions:

    src/devices/         printers and scales, the stock a printer is loaded with, and the
                         station that offers them
    src/nodes/           what a node keeps about the other nodes it works with
    src/namespaces.rs    the namespaces and their models
    src/seed.rs          seeding a node with the kept mint, which it compiles in
    mint/                the tool that mints the definitions and checks a mint, for this
                         crate and for any other crate of Koinon models

The Rust declarations are the source: each `#[derive(Model)]` struct states a model, its
properties, how each is stored and which are optional, and names the id the kept mint gives
the model. Every model is marked `no_ffi`, as the Ankurah Commons' are.

A comment that starts with `OPEN:` marks a question still to settle. Settling one that changes
a model changes its definitions, which then need minting again.

## One entity, several components

A model is an Ankurah component, and one entity may carry several:

    a station    Station, later Closed when retired
    a printer    Device + Printer, with Media once someone has said what is loaded, later
                 Closed when retired
    a scale      Device + Scale, later Closed when retired
    a sign-in    NodeLink, later Closed when signed out, and reopened when signed in to
                 the same node again

`Closed` is the Ankurah Commons' own, from its `records` namespace. A station closes a
record with `by` unset, because it has no principal of its own to name.

A station is a node that offers the printers and scales on its computer, such as an
installed copy of Pylon, the app through which business software prints and weighs on the
devices beside a computer. It makes the records of the `devices` namespace and sends them
to every server it is signed in to, so that each server holds the same records under the
same ids. A `NodeLink` stays on the node that made it.

A station describes a printer as its computer's print system does: the name of its driver,
and whether the print system renders a PDF for it. What the printer is loaded with is what
a person last said, written whole rather than as a reference, so that it reads the same on
every server. A width or a height is a `Dimension`, the Ankurah Commons' type.

## Building against Ankurah

The crate builds against the revision of Ankurah's main that the Ankurah Commons pins,
named in `Cargo.toml`. An application that uses the crate pins the same revision.

## Publishing

A definition's id is the hash of its creation event, and a record's memberships in models
are written into its own events by those ids. So a record a station makes is a device on a
server only if both nodes hold the same definitions, and publishing a model means minting
its events once and keeping what was minted, as the Ankurah Commons does for its own.

`mint/` does this, through the Ankurah Commons' `registration` and `akev`:

    cargo run -p koinon-mint -- mint [--src <dir>]
    cargo run -p koinon-mint -- verify [--src <dir>]

`mint` writes the creation event of every definition each model's declaration needs, as a
block of the `.akev` file beside the struct, and gives the struct its model's id,
`#[model(id = "...")]`. An event the kept mint holds with the same operations keeps its
nonce and timestamp, so minting again changes nothing unless a declaration changed, and then
draws new events only for the definitions whose operations changed. A model keeps its id
unless its label or the name of its struct changes; a changed property gives that property
and its link new ids. `verify` checks that each file states
exactly what its model's declaration needs and that each struct declares its file's id.
Both work on the namespaces' parent directory, `src/` unless `--src` names a copy for a
trial run.

A mint that is kept is committed to this repository, and that text is then the authority.
Each creation event names no system root and no author, so it is the same entity on every
node. Nothing is signed: what vouches for the events is this repository. The ids hold for
the event format of the Ankurah revision the crate builds against; a change to that format
needs the kept mint carried over, as the Ankurah Commons' mint describes.

`src/namespaces.rs` lists the models `mint` mints, and a test there fails when a namespace's
directory declares a model the list does not name. A model declared anew is listed, minted,
and its file added to the list in `src/seed.rs`, which the seed's tests check.

The tool mints any crate of Koinon models that lists its namespaces as the Ankurah Commons
lists its own, as `ankurah_commons::namespaces::Namespace`, with no kinds of edge. The declarations are compiled in, so such a crate has a binary of
its own that calls `koinon_mint::run` with its list.

## Seeding

`seed` imports the kept mint into a durable node, through Ankurah's `Node::import_events`,
so that the models bind to the minted definitions instead of minting their own. A node seeds
the Ankurah Commons first, whose components the entities of these models also carry, then
the Koinon Commons:

    let node = Node::new_durable(Arc::new(engine), agent);
    node.system.create().await?; // a fresh store; on one that holds a root, wait_system_ready()
    ankurah_commons::seed(&node).await?;
    koinon_commons::seed(&node).await?;

Seed the node before it first binds any of these models: registering a model binds it, and
so do a fetch, a query, a get and a create of it. Each model declares its minted id and binds
by that id, never by its label. On a node not yet seeded it fails with Ankurah's
`RegistrationError::ExplicitModelIdNotFound` rather than minting definitions of its own,
which no other node would share.

Seeding a store that holds the kept mint already changes nothing, so a node can be seeded at
every start.

## Licence

MIT or Apache-2.0, at your option: `LICENSE-MIT` and `LICENSE-APACHE`.
