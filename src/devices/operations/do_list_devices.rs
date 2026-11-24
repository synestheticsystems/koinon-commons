use ankurah::{envelope::EnvelopeComponent, Model};

/// Asks a station which of its devices the request's authority reaches, so that a party learns
/// the ids of the devices it may use with no registry to ask. Beside `Request`, it requires an
/// authority, as `authority` sets out: the request signed whole by its application, or an
/// `AuthToken` presenting a permission slip. What either reaches is the station's approval of the
/// application, narrowed by the slip when there is one: a slip that allows any operation lets its
/// party list the devices, and the listing names those of the kinds whose operations it allows.
///
/// It is answered with `Response` and a `DeviceListing` with a row for each device the authority
/// reaches, and may add the station's own record. A device's `Device` travels whole, its
/// `system_name` with it: a sender leaves out a whole component and never one of its properties. A
/// slip that covers devices none of which this station offers is refused as `not_authorized`,
/// rather than answered with no device.
///
/// A receiver must understand it, so that one that does not know it refuses the request, naming
/// it.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq)]
#[model(label = "devices.do_list_devices")]
#[model(id = "WSvMYraudMlvE9bRbsZ9flcDaUl9TY7vjrlW85CwU4o")]
#[model(no_ffi)]
#[envelope(must_understand)]
pub struct DoListDevices {}
