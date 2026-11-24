use ankurah::{envelope::EnvelopeComponent, Model};

use crate::component_ids::ComponentIds;

/// Names every component of a refused request, flagged as one a receiver must understand, that the
/// side asked does not understand, so that the sender learns in one answer what it may leave out,
/// or that the side it asked is too old for what it asked. It comes beside a `Refusal` whose code is
/// [`crate::replies::Refusal::NOT_UNDERSTOOD`].
// OPEN: how such a refusal names the request it answers. The side asked cannot read the request's
// `Request` while the request holds a component flagged must-understand that it does not know:
// every read of Ankurah's envelope refuses then, and none leaves the flags to the reader. Either
// the envelope gains a read that does, or a refusal of this kind holds no `Response`, and its
// asker knows it by the stream it came on, as the messages' tests now do.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq)]
#[model(label = "replies.not_understood")]
#[model(id = "41_X-OqloSwZ1O0EaXbFmy7owalg4EvwnJld2tIUbT0")]
#[model(no_ffi)]
pub struct NotUnderstood {
    #[active_type(LWW)]
    #[property(id = "EmwMiA69orwdbYltfP0vGTsLNtft7AoactXdFozmkRA")]
    #[envelope(number = 0)]
    pub components: ComponentIds,
}
