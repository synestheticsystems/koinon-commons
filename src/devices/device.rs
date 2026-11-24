use super::Station;
use ankurah::{envelope::EnvelopeComponent, property::Ref, Model};
use serde::{Deserialize, Serialize};

/// One printer or scale that a station offers. What kind it is follows from the other
/// components on its entity: `Printer`, with `Media` once someone has said what it is
/// loaded with, or `Scale`. Its retirement is the Ankurah Commons' `Closed`.
///
/// Whether the device is ready at this moment is not recorded: the station reports that
/// over its connection.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[model(label = "devices.device")]
#[model(id = "ULVl4nlCm9s4wkRAosvq9iDNwy5ePclWoGHd_c60z-g")]
#[model(no_ffi)]
pub struct Device {
    /// The station whose computer the device is on, which carries out the jobs for it. It
    /// never changes: a device moved to another computer is added there as a new record.
    #[property(id = "oonHtZVf7XkOnRruznuVrlnOlYVg0oRRgopPdDFYTEM")]
    #[envelope(number = 0)]
    pub station: Ref<Station>,
    /// What that computer's operating system calls the device, by which the station
    /// finds it again: a print queue's name, or what tells a scale from the others on that
    /// computer, such as its serial number.
    #[active_type(LWW)]
    #[property(id = "F376lqdzGksKqfAKSi4N6yFvX5dyp1GRC1vyhpfSGXU")]
    #[envelope(number = 1)]
    pub system_name: String,
    /// What a person called it, such as "Pack bench 3 labels".
    #[active_type(LWW)]
    #[property(id = "jkm85rInO7Xjr4OOVI9oYb2vgey49mlm7k3jFJFVVq8")]
    #[envelope(number = 2)]
    pub name: String,
}
