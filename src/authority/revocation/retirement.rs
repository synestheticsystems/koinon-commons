use ankurah::Model;
use ankurah_commons::property::Timestamp;

use crate::codec::Minted;

/// An application's word that its key is retired: how whoever holds the key, such as the
/// application's registry once the key has leaked, stops every station trusting it without
/// waiting on each station's person. It travels inside a `SignedRetirement`, whose signer is the
/// application's key, and so it proves itself and may come by any path.
///
/// A station that shares its devices with the application closes its record of the application
/// from the moment it hears of this, refuses the key from then on by every path, and refuses to
/// share with the key again. A station keeps nothing of the retirement of an application it does
/// not share with, so that no one can fill it with retirements of keys made for the purpose. A
/// sender lists it, inside the signed envelope, as a component a receiver must understand.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "authority.retirement")]
#[model(id = "a0fDwfoJf36ur9V6YJUnV7haIFSC_ocuR8QXhE6vicQ")]
#[model(no_ffi)]
pub struct Retirement {
    /// When the key was retired: what a station's window says of when the application lost its
    /// access.
    #[property(id = "bFmPN426_nlcXjIIwtFCBie8jl3e-l8muDXJh2jzwqg")]
    pub retired: Timestamp,
}
