use ankurah::{
    envelope::{EntityRows, EnvelopeComponent},
    Model,
};

/// The registrations a registry keeps for a station: the answer to a `DoListRegistrations`. A row
/// for each registration, under no entity id, holds its `Registration`, with the grant it is
/// under, and the application's `ForApplication`, with `SlipsVoided` when the registry has voided
/// the application's slips: the latest voiding, which the station applies as it applies the same
/// news on any other path.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq)]
#[model(label = "registry.kept_registrations")]
#[model(id = "TGzFu8EhZGv6pU4LQGfw5cxmcsxxX51qx2fU5qAqZvg")]
#[model(no_ffi)]
pub struct KeptRegistrations {
    /// A row for each registration, in no particular order.
    #[active_type(LWW)]
    #[property(id = "6xnEjI-8qGCTv007XZkp2-T0eIR5cCV-7gN6AT3MUVg")]
    #[envelope(number = 0)]
    pub registrations: EntityRows,
}
