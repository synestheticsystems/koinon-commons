use ankurah::Model;

use crate::codec::Minted;

/// That the side asked did what it was asked, and has nothing more to tell: the answer to a
/// request that asks for nothing back, such as a registration, a push of events, or a notice that
/// a station no longer shares its devices with an application.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "replies.done")]
#[model(id = "uD9Ql4NvG2qHoePa1uYUeH9HAHk3LY5MTfgqJfjeDTk")]
#[model(no_ffi)]
pub struct Done {}
