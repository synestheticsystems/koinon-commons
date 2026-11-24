use ankurah::{envelope::EnvelopeComponent, Model};

use super::HostName;
use crate::{name::Name, public_key::PublicKey};

/// That a registry hosts the application whose key signs this, under a name, for the people who
/// log in at an address. An envelope of it alone, signed by the application's key, is the
/// application's endorsement of the registry's listing of it, which a registry serves in its
/// document over HTTPS, as that envelope's exact bytes, and from which alone a station takes the
/// application's key, name and address before it asks its person anything. It binds all three, so
/// that no registry can list a key it does not serve, nor give the application another name or
/// another address; and a station takes it only from the document of the registry it names. A
/// receiver must understand it.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq)]
#[model(label = "authority.hosting")]
#[model(id = "AX3KgpiVAyAjTj-_hzmYIUdNx9ZB2g4ndHP5O7JBnYY")]
#[model(no_ffi)]
#[envelope(must_understand)]
pub struct Hosting {
    /// The key of the registry's iroh endpoint, which a station dials: the one registry the
    /// statement vouches for.
    #[active_type(LWW)]
    #[property(id = "p2dAQ1-mOg0ns-fpJXDsh2Lg6QN1uta2evKYogPj7DA")]
    #[envelope(number = 0)]
    pub registry: PublicKey,
    /// What people call the application, such as "Acme": the name a station's question shows
    /// its person.
    #[active_type(LWW)]
    #[property(id = "EFgo84PrhdH_N1ACrt84_zvIzdf11QG76og_dMKxJkg")]
    #[envelope(number = 1)]
    pub name: Name,
    /// The application's address, where its people log in: its host name alone, in the one form
    /// in which a host name is written, such as `acme.example.com`, so that a registry and a
    /// station write one address one way. A station compares it, as text, with the address its
    /// person typed, which it writes in that form first ([`HostName::from_typed`]), through the
    /// same parser. A hosting whose address is written any other way, or is an IP address, does
    /// not read.
    #[active_type(LWW)]
    #[property(id = "0CKENjMLxBy4vNsWIqvfUL5vAGZJBTkJ4ubBxg3UqK8")]
    #[envelope(number = 2)]
    pub address: HostName,
}
