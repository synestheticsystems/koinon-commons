use ankurah::Model;

use crate::{codec::Minted, name::Name, public_key::PublicKey};

/// That a registry hosts the application whose key signs this, under a name, for the people who
/// log in at an address: the statement a `SignedHosting` carries, which its envelope lists as a
/// component a receiver must understand. It binds all three, so that no registry can list a key it
/// does not serve, nor give the application another name or another address.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "authority.hosting")]
#[model(id = "AX3KgpiVAyAjTj-_hzmYIUdNx9ZB2g4ndHP5O7JBnYY")]
#[model(no_ffi)]
pub struct Hosting {
    /// The key of the registry's iroh endpoint, which a station dials: the one registry the
    /// statement vouches for.
    #[active_type(LWW)]
    #[property(id = "p2dAQ1-mOg0ns-fpJXDsh2Lg6QN1uta2evKYogPj7DA")]
    pub registry: PublicKey,
    /// What people call the application, such as "Acme": the name a station's question shows
    /// its person.
    #[active_type(LWW)]
    #[property(id = "EFgo84PrhdH_N1ACrt84_zvIzdf11QG76og_dMKxJkg")]
    pub name: Name,
    /// The application's address, where its people log in: its host name alone, in lower case,
    /// with an internationalized name in its ASCII form (`xn--`), and with no scheme, port or
    /// path, such as `acme.example.com`, so that a registry and a station write one address one
    /// way, and a station compares it with the address its person typed.
    #[active_type(LWW)]
    #[property(id = "0CKENjMLxBy4vNsWIqvfUL5vAGZJBTkJ4ubBxg3UqK8")]
    pub address: String,
}
