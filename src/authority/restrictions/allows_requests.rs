use ankurah::Model;

use crate::{codec::Minted, component_ids::ComponentIds};

/// The kinds of request a permission slip lets its party make, each by its request component's
/// minted id, such as `Print`'s or `Weigh`'s, so that a kind of request added later needs nothing
/// new here. Each kind also lets the party list the devices of its kind, with `ListDevices`. A
/// station refuses as `not_authorized` a request whose component the slip does not list. Every
/// slip has it.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "authority.allows_requests")]
#[model(id = "Jdn2IDt0Pgq_BnOR7WmD1CSzJrdYFU4f5T41DHGlMuo")]
#[model(no_ffi)]
pub struct AllowsRequests {
    #[active_type(LWW)]
    #[property(id = "Ylb5QfNpqOGpQUz-RsMxL4X-UdI9he8Oq4IB5AexBsE")]
    pub requests: ComponentIds,
}
