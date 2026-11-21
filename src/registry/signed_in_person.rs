use ankurah::Model;

use crate::codec::Minted;

/// Who signed in, as their ID token names them: for the station to show the person whose yes it
/// asks for, and to record who said it. A token may give neither a name nor an address, which is
/// why the Ankurah Commons' `Principal`, whose handle and name are required, does not say it; the
/// station makes a principal for the person from this.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "registry.signed_in_person")]
#[model(id = "pnldNpll2AomAhSTRpQAVKtLmFl7HL6xUXgeH_mdXeo")]
#[model(no_ffi)]
pub struct SignedInPerson {
    /// The token's `name`. None when the provider gave none.
    #[active_type(LWW)]
    #[property(id = "i_6psTpvLMUVdI2r9oOZwwHh-r4qCs1A2Le-XoSEk_Y")]
    pub name: Option<String>,
    /// The token's `email`. None when the provider gave none.
    #[active_type(LWW)]
    #[property(id = "Ymr6H0veuUiN6UrTdnslYtIUpV0m7GXuiSy_ikOs-h0")]
    pub email: Option<String>,
}
