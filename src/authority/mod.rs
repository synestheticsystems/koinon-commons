//! Who may ask a station for what, and how a request proves it, so that a station decides by
//! itself, with no registry, relay or internet involved, and a job reaches it from anyone its
//! person trusts, from wherever they are.
//!
//! An application is a named key pair that a station's person chose to share the station's
//! devices with: one that a registry hosts, which the person said yes to after signing in there,
//! or a computer the person paired with the station directly, whose own key is then the
//! application. A request has its authority from the application's key, in one of two ways and
//! never both:
//!
//! - signed whole: the request's own envelope carries the signature of the application's key,
//!   and holds `AtStations`, naming this station alone, and `Expiry`, which
//!   [`check_signed_request`] checks;
//! - under a permission slip: the request holds an `AuthToken` presenting a slip the application
//!   signed for the party whose key the request's connection proves, which [`open_slip`] and
//!   [`check_slip`] check.
//!
//! What a request reaches is set out here once, for every request that names a device and for a
//! listing of devices. A request reaches the devices that the station's approval of its
//! application lets it use, and no others: every device the station offers, for an application
//! that came through a registry, and those its person offered, for a computer they paired with it.
//! A request under a permission slip reaches those of them that the slip also covers, and only
//! those of the kinds whose operations the slip allows: printers for `DoPrint`, scales for
//! `DoWeigh` ([`slip_covers`]). A station refuses as `not_authorized` a request for a device its
//! authority does not reach, and lists no such device in answer to a `DoListDevices`.
//!
//! Here are the applications a station shares its devices with, and the two ways it comes to;
//! permission slips, the parties they are for, and the checks of a slip; the restrictions on
//! where, for what and until when an authority is honoured, which a receiver must understand; how
//! an application's authority ends before it lapses; and the checks of a request signed whole.

mod applications;
mod restrictions;
mod revocation;
mod signed_requests;
mod slips;

pub use applications::*;
pub use restrictions::*;
pub use revocation::*;
pub use signed_requests::*;
pub use slips::*;
