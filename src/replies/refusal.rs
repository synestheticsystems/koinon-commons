use ankurah::Model;

use crate::codec::Minted;

/// That the side asked did not do what it was asked, and why: a code for a program to act on, and
/// words for a person. A registry passes a station's refusal on to the caller that sent the request,
/// as the error of its API with the same code. A job that was taken and failed is not refused: its
/// `JobFailure` says how it failed.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "replies.refusal")]
#[model(id = "L3fpHa-JGkQ0wIdz1gmk7EM7jUXc392omNcylGZqkpI")]
#[model(no_ffi)]
pub struct Refusal {
    /// One word in snake case, such as [`Refusal::NOT_FOUND`]. An asker that does not know the
    /// word takes the refusal as one it cannot act on, and shows the message.
    #[active_type(LWW)]
    #[property(id = "1sjAb304gYogFxsCb4A7OfHqW1dJVOICvjzX8UEPxxI")]
    pub code: String,
    /// What a person reads of the refusal.
    #[active_type(LWW)]
    #[property(id = "HGf6j32RDAz4cAGIXvpy9n2nAHMtcuM27QgLURf6BpE")]
    pub message: String,
}

impl Refusal {
    /// The asker must be signed in for what it asked, and is not: the registry holds no grant for
    /// the station's key, or accepted no sign-in on this connection.
    pub const NOT_SIGNED_IN: &'static str = "not_signed_in";
    /// The ID token of a sign-in fails the registry's checks of its signature, issuer, audience,
    /// expiry and nonce, as one issued for a sign-in to another registry does, or the registry
    /// has accepted it once already. The person signs in again.
    pub const CREDENTIAL_REFUSED: &'static str = "credential_refused";
    /// The asker is known, and may not ask this, or the station honours no authority the request
    /// carries.
    pub const NOT_AUTHORIZED: &'static str = "not_authorized";
    /// The request names a device, a job or an application that the side asked does not hold.
    pub const NOT_FOUND: &'static str = "not_found";
    /// The device is known, and cannot take the work now, such as a printer whose print system
    /// refuses work or a scale that is not plugged in.
    pub const DEVICE_UNAVAILABLE: &'static str = "device_unavailable";
    /// The request lacks a component it requires, or breaks a rule of its request.
    pub const BAD_REQUEST: &'static str = "bad_request";
    /// The device cannot do what was asked: a print on a scale, a weigh on a printer, or a
    /// document in a format that the printer's `document_formats` do not list.
    pub const UNSUPPORTED: &'static str = "unsupported";
    /// The side asked failed at its own part, such as writing to its store, so the request may or
    /// may not have taken effect. The asker repeats it as it was, under the same id.
    pub const INTERNAL_FAILURE: &'static str = "internal_failure";
    /// The request lists as ones a receiver must understand components the side asked does not
    /// understand, which a `NotUnderstood` beside the refusal names.
    pub const NOT_UNDERSTOOD: &'static str = "not_understood";
}
