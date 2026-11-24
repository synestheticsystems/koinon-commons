use ankurah::{envelope::EnvelopeComponent, Model};

use crate::component_ids::ComponentIds;

/// The operations a permission slip lets its party ask for, each by its tag's minted id, such as
/// `DoPrint`'s or `DoWeigh`'s, so that an operation added later needs nothing new here. A station
/// refuses as `not_authorized` a request for an operation the slip does not list, but for a
/// `DoListDevices`: a slip that allows any operation lets its party list the station's devices,
/// and the listing names those of the kinds whose operations it allows, printers for `DoPrint` and
/// scales for `DoWeigh`. Every slip has it.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq)]
#[model(label = "authority.allows_requests")]
#[model(id = "Jdn2IDt0Pgq_BnOR7WmD1CSzJrdYFU4f5T41DHGlMuo")]
#[model(no_ffi)]
#[envelope(must_understand)]
pub struct AllowsRequests {
    #[active_type(LWW)]
    #[property(id = "Ylb5QfNpqOGpQUz-RsMxL4X-UdI9he8Oq4IB5AexBsE")]
    #[envelope(number = 0)]
    pub requests: ComponentIds,
}
