//! The applications a station shares its devices with, each named by its key, and the two ways a
//! station comes to share them: through a registry that hosts the application, as the
//! application's key endorses, or by a computer's claim of an invitation the station's person
//! made, which makes the computer's own key an application that no registry keeps.

mod for_application;
mod hosting;
mod invitation_claim;
mod invitation_secret;

pub use for_application::*;
pub use hosting::*;
pub use invitation_claim::*;
pub use invitation_secret::*;
