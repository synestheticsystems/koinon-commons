use ankurah::Model;

use crate::{codec::Minted, name::Name, public_key::PublicKey};

/// The party a permission slip is for: the key of its iroh endpoint, which its connection to a
/// station proves, so that the slip is of use to its party alone, and what the person who asked
/// for the slip calls it, so that a station's window can list the party and its person can tell
/// whom to block. A slip lists it as a component a receiver must understand, since a station that
/// dropped it would honour the slip for whoever presented it.
///
/// On a station's records, beside its application's `ForApplication`, it is the station's record
/// of a party it took jobs from under the application's slips, as the latest slip the party
/// presented named it. Blocking the party is the Ankurah Commons' `Closed`, on the same entity.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "authority.party")]
#[model(id = "enZdC2C2ri1IUqNd1JFiQ5Ndq_CuxFFx7Bjtwmp5Svs")]
#[model(no_ffi)]
pub struct Party {
    /// The key the party's connection to a station proves.
    #[active_type(LWW)]
    #[property(id = "r2TCNcuAcInpFWlfFpB8CVMbUrMxohSBbxlSDvQ7tLw")]
    pub key: PublicKey,
    /// What the person who asked for the slip calls the party, such as "Warehouse label PC". A
    /// station shows it and checks nothing by it.
    #[active_type(LWW)]
    #[property(id = "v6uFglx7KyVRHcMdfDV6wMswpZyZARsDNdb2jTlmL7A")]
    pub name: Name,
}
