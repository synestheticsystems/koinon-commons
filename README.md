# Koinon Commons

Definitions of the Ankurah models that Koinon programs share, so that two programs, or two
nodes, mean the same thing by "a device" or "a station". A layer on the
[Ankurah Commons](https://github.com/ankurah/commons), in its shape.

**Status: draft, with a mint kept, and no id final.** The definitions of every model in the
seven namespaces, 50 models with 67 properties and their links, are minted, unsigned, in the
`.akev` files beside the structs, in the format of the Ankurah revision this crate builds
against.

## Layout

One crate, `koinon-commons`, with a module per namespace, one file per model, and beside each
model's file the `.akev` file of its minted definitions:

    src/authority/       who may ask a station for what, and how a request proves it: the
                         application a request is for, permission slips, the restrictions on
                         an authority, the carriers of what an application's key signed, and
                         how an authority ends before it lapses
    src/catalog/         what a registry knows of printers
    src/devices/         printers and scales, a stock of the kind a printer is loaded with, the
                         station that offers them, and what a scale reads
    src/jobs/            what a station is asked to do on its devices, and where each job
                         stands
    src/nodes/           what a node keeps about the other nodes it works with, and the
                         events of its own records it hands them
    src/registry/        a registry's scopes and grants, and how a station signs a person in
                         there and registers for an application
    src/replies/         what any reply may say of its request: done, refused, or not
                         understood
    src/mime_type.rs     MIME types, by which a document names its format and a printer the
                         formats it takes
    src/public_key.rs    the keys by which components name stations, parties and applications
    src/component_ids.rs lists by which one component names others, by their minted ids
    src/namespaces.rs    the namespaces and their models
    src/seed.rs          seeding a node with the kept mint, which it compiles in
    mint/                the tool that mints the definitions and checks a mint, for this
                         crate and for any other crate of Koinon models

The Rust declarations are the source: each `#[derive(Model)]` struct states a model, its
properties, how each is stored and which are optional, and names the ids the kept mint gives
the model and each of its properties. Every model is marked `no_ffi`, as the Ankurah Commons' are.

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
    a scope      Scope, with its parent when it is a sub-scope, later Closed when it ends
    a station's  ScopeAssignment + Placement, later Closed when it ends
    assignment
    a grant      NodeGrant, with ForApplication for the application it is for, later
                 Closed when revoked or signed out

`Closed` is the Ankurah Commons' own, from its `records` namespace. A station closes a
record with `by` unset, because it has no principal of its own to name.

A station is a node that offers the printers and scales on its computer, such as an
installed copy of Pylon, the app through which business software prints and weighs on the
devices beside a computer. It makes the records of the `devices` namespace and sends them
to every server it is signed in to, so that each server holds the same records under the
same ids. A `NodeLink` stays on the node that made it.

A station describes a printer as its computer's print system does: the name of its driver,
and the formats of document the print system accepts for it, as MIME types. What the printer is loaded with is what
a person last said, written whole rather than as a reference, so that it reads the same on
every server. A width or a height is a `Dimension`, the Ankurah Commons' type.

## Messages

Every message on Pylon's iroh connections is a set of these components, in an envelope of
`codec`. A component means the same in a message as on an entity, so that every sender of one
meaning sends the one component that means it, and a record can keep a component as a message
carried it. Components are filed by what they mean, never by whether they travel.

- A request is an envelope that holds one request component, which has no properties: it says
  what is asked, and the facts it acts on travel beside it. A namespace's `requests` module
  holds the requests that act on its records, such as `jobs::requests::Print`, and each
  request's documentation is its contract: what it requires and takes, what answers it, and
  what refuses it.
- A reply comes back on its request's stream. It is a refusal when it holds `Refusal`, with
  `NotUnderstood` when the receiver did not understand a component it had to; otherwise it is
  the answer, the components its request names, or `Done`.
- A list of things that each hold several components travels as the envelope's records, such
  as a station's devices or a registry's catalog.
- What an application's key signs travels as the exact bytes it signed, in a carrier of
  `authority`: a request signed whole, the permission slip a party's request comes under, or
  the key's retirement. A receiver checks the signature over the bytes as they arrived, then
  reads them, and acts on nothing else.
- A sender lists as one a receiver must understand each request component, each carrier, and
  each restriction an older receiver must not drop, such as an `Expiry`. A restriction added
  later is a component of its own, listed so, and never a new property of an old one, which an
  older receiver would ignore.

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
block of the `.akev` file beside the struct. It gives the struct its model's id,
`#[model(id = "...")]`, and every field that declares a property that property's id,
`#[property(id = "...")]`, so that a node binds a model and each of its properties by their
ids, never by label or name. An event the kept mint holds with the same operations keeps its
nonce and timestamp, so minting again changes nothing unless a declaration changed, and then
draws new events only for the definitions whose operations changed. A model keeps its id
unless its label or the name of its struct changes; a changed property gives that property
and its link new ids. `verify` checks that each file states exactly what its model's
declaration needs and that each struct declares its file's ids, its model's and each
property's. Both work on the namespaces' parent directory, `src/` unless `--src` names a copy
for a trial run.

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
so do a fetch, a query, a get and a create of it. Each model and each of its properties
declare their minted ids and bind by them, never by label or name. On a node not yet seeded it fails with Ankurah's
`RegistrationError::ExplicitModelIdNotFound` rather than minting definitions of its own,
which no other node would share.

Seeding a store that holds the kept mint already changes nothing, so a node can be seeded at
every start.

## Licence

MIT or Apache-2.0, at your option: `LICENSE-MIT` and `LICENSE-APACHE`.
