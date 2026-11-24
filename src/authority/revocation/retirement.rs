use ankurah::{envelope::EnvelopeComponent, Model};
use ankurah_commons::property::Timestamp;

/// An application's word that its key is retired: how whoever holds the key, such as the
/// application's registry once the key has leaked, stops every station trusting it without
/// waiting on each station's person. It is a request of its own, which needs no tag: an envelope
/// of `Request` and this, signed by the key it retires, so that it proves itself and may come by
/// any path. Whoever holds it passes it on as those exact bytes, its request's id with them: a
/// registry down its held connection to each station registered for the application, and anyone on
/// the direct path.
///
/// A station that shares its devices with the application closes its record of the application
/// from the moment it hears of this, refuses the key from then on by every path, and refuses to
/// share with the key again, by either way: through a registry, or by an `InvitationClaim`. A
/// station keeps nothing of the retirement of an application it does not share with, so that no
/// one can fill it with retirements of keys made for the purpose. It is answered with `Response`
/// alone, whether or not the station shares its devices with the application, so that the answer
/// tells no one which applications it shares with.
///
/// A receiver must understand it, so that one that does not know it refuses the request, naming
/// it.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq)]
#[model(label = "authority.retirement")]
#[model(id = "a0fDwfoJf36ur9V6YJUnV7haIFSC_ocuR8QXhE6vicQ")]
#[model(no_ffi)]
#[envelope(must_understand)]
pub struct Retirement {
    /// When the key was retired: what a station's window says of when the application lost its
    /// access.
    #[property(id = "bFmPN426_nlcXjIIwtFCBie8jl3e-l8muDXJh2jzwqg")]
    #[envelope(number = 0)]
    pub retired: Timestamp,
}
