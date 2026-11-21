use ankurah::{property::Ref, Model};

use crate::codec::Minted;

/// A scope, or a sub-scope of one: whose stations, people, keys and jobs a record belongs with. A
/// registry makes it; a station knows of a scope only what a registry's answer to a sign-in tells
/// it.
///
/// Who made it is the Ankurah Commons' `Created`, and its end `Closed`, on the same entity.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "registry.scope")]
#[model(id = "8T5vX5X1wScs72KInBtY-jdWKJC_2-RQpOxpvWqJNLk")]
#[model(no_ffi)]
pub struct Scope {
    #[active_type(LWW)]
    #[property(id = "cGZRnB6_zhCfjla1PLfeurcxz3JHTIJvntzxV8W-Evg")]
    pub name: String,
    /// The scope this is a sub-scope of. None on a scope. It never changes, and a sub-scope has
    /// no sub-scopes of its own: a `Placement` names a scope and one of its sub-scopes, and
    /// nothing deeper.
    #[active_type(LWW)]
    #[property(id = "b_snDbbUJ9tZPd28QN_sT_u0ATJRQV6l1KciEagUjWk")]
    pub parent: Option<Ref<Scope>>,
}
