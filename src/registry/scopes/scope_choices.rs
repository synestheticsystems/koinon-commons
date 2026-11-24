use ankurah::{
    envelope::{EntityRows, EnvelopeComponent},
    Model,
};

/// The scopes and sub-scopes a person who signed in may register the station in, for the station
/// to show its person: the answer to an `IdTokenSignIn`, beside `SignedInPerson`. A row for each,
/// under its entity id, holds `Scope` and `Registrable`. Where the person may register only in
/// some of a scope's sub-scopes, that scope's row holds `Scope` alone, so that the station can
/// show the sub-scopes under its name. A person who is a member of no scope gets no row, and may
/// name a `FirstScope` instead.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq)]
#[model(label = "registry.scope_choices")]
#[model(id = "OCIUOsGa0tCztwDspQg-8H8afxpXx70BhY_FSkvd7KY")]
#[model(no_ffi)]
pub struct ScopeChoices {
    /// A row for each scope and sub-scope, each sub-scope after its scope.
    #[active_type(LWW)]
    #[property(id = "XtvTVY8rLMuDSwEp6F2mNDZ-bCCSqnnk4ehEwI-y3kE")]
    #[envelope(number = 0)]
    pub scopes: EntityRows,
}
