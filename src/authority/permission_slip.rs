use ankurah::{EntityId, Model};
use ankurah_commons::property::Timestamp;

use crate::{codec::Minted, public_key::PublicKey};

/// The body of a permission slip: the application whose key signs the slip lets the party with
/// this key make the requests the slip allows, where and until the slip says. It is how an
/// application's authority reaches a program that does not hold the application's key, such as a
/// program on a warehouse's own network, while the key stays with whoever holds it, such as the
/// application's registry, which signs a slip when a person, or a caller acting for the
/// application, asks for one. It travels inside a `PresentedSlip`, whose signer is the
/// application, beside the restrictions that say where, for what and until when the slip holds:
///
/// - `AtStations`, required: the stations at which the slip holds, which tell the party where to
///   dial;
/// - `AtDevices`, optional: the devices at those stations to which the slip narrows;
/// - `AllowsRequests`, required: the kinds of request the party may make;
/// - `Expiry`, required: until when it holds, at most [`PermissionSlip::MAX_LIFE_MS`] after its
///   issue.
///
/// Every restriction present must hold. A slip that lacks `AtStations`, `AllowsRequests` or
/// `Expiry` is refused. A restriction that is optional and absent sets no limit of its own, so a
/// restriction added later is optional and listed as one a receiver must understand: it can only
/// narrow what a slip allows. The slip lists this component and every restriction so.
///
/// A station honours a slip while it shares its devices with the slip's application, and only one
/// issued since its person last said yes to sharing with the application, and since the
/// application's registry last voided its slips (`SlipsVoided`). It refuses a slip issued later
/// than now, by more than the drift of clocks that `Expiry` allows.
// OPEN: whether an application that a station's person paired with directly, with no registry,
// may sign slips. The rules above hold either way; a station's check of the slip's signer names
// the applications that may.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "authority.permission_slip")]
#[model(id = "sW4D0xcq9TVaWSu6kyop7gXpHdCQcBjzagQOBaa_LYs")]
#[model(no_ffi)]
pub struct PermissionSlip {
    /// The party the slip is for, by the key of its iroh endpoint, which its connection to a
    /// station proves, so that the slip is of use to its party alone.
    #[active_type(LWW)]
    #[property(id = "nlPHmwqASvnvBR0UmNuQxX3aws88yy6BM5NK1Vkn0wE")]
    pub party: PublicKey,
    /// What the person who asked for the slip calls its party, such as "Warehouse label PC": the
    /// name a station's window lists the party under, so that its person can tell whom to block.
    #[active_type(LWW)]
    #[property(id = "-oVKJAnlW1ftmSbZI7glph-u5xyJBELl8QytALUcP9c")]
    pub party_name: String,
    /// When the slip was issued.
    #[property(id = "qjlXIUr2HftQP9Kf6lwBKiguJcEiczvr1VijjU5lWx0")]
    pub issued: Timestamp,
    /// The slip's id, which no other slip of the application has: on a registry, the id of its
    /// record of the slip. A station's journal names by it the slip each job came under, so that a
    /// job can be traced to the slip and to whoever asked for it.
    #[property(id = "tcjLiCa1HVdnOEGYOGO8ycyPRiDc6wD57s23SN8wjvw")]
    pub slip_id: EntityId,
}

impl PermissionSlip {
    /// The longest a slip is honoured for, in milliseconds from its issue: thirty days. A station
    /// refuses a slip whose expiry lies further from its issue, so that a slip a station never
    /// hears voided still lapses within a month.
    pub const MAX_LIFE_MS: i64 = 30 * 24 * 60 * 60_000;
}
