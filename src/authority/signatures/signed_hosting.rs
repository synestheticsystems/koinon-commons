use ankurah::Model;

use super::{Signature, Signed};
use crate::{
    codec::{Minted, SealedEnvelope},
    public_key::PublicKey,
};

/// An application's endorsement of a registry's listing of it, as the application's own key signed
/// it: that the registry hosts the application under a name, for the people who log in at an
/// address, as its `Hosting` says. A registry lists each application it hosts by its endorsement,
/// in the document it serves over HTTPS, as the three parts the key signed, and a station takes the
/// application's key, name and address from the endorsement alone, its signer and the bytes it
/// signed, before it asks its person anything. The application's operator signs one for each
/// registry that hosts it, and again when the name or the address changes.
///
/// Its envelope carries `Hosting`, listed as a component a receiver must understand, and
/// the application it speaks for is the signer. A station opens it ([`Signed::open`]) as a reader of
/// hostings, which understands `Hosting` and nothing else, then takes it only when its `Hosting`
/// names the registry whose document listed it.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "authority.signed_hosting")]
#[model(id = "vb5dzEeChBzokDcK2Ugn2Gz6Yxq5NZS6DkUy9TDqg2o")]
#[model(no_ffi)]
pub struct SignedHosting {
    /// The hosting's envelope, as the application's key signed it.
    #[property(id = "xw9ja_uy5wtzExttj8M_F3jCdRL9TsW0mBqFfjzZaPw")]
    pub content: SealedEnvelope,
    /// The application's key.
    #[active_type(LWW)]
    #[property(id = "QsqeqZ8LQgZfQsd2crnkmuZwPwfPa1c4EUHftwbpZBs")]
    pub signer: PublicKey,
    #[property(id = "L_lAjIlk4HS3tqvac-uLqc0RmJLGmPpeYradDzPRJyg")]
    pub signature: Signature,
}

impl Signed for SignedHosting {
    fn new(content: SealedEnvelope, signer: PublicKey, signature: Signature) -> Self {
        SignedHosting { content, signer, signature }
    }

    fn content(&self) -> &SealedEnvelope {
        &self.content
    }

    fn signer(&self) -> &PublicKey {
        &self.signer
    }

    fn signature(&self) -> &Signature {
        &self.signature
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        authority::{HostName, Hosting},
        codec::{EnvelopeBuilder, Limits},
        name::Name,
    };
    use ed25519_dalek::SigningKey;

    // What a registry's document lists an application by, as a station reads it before it asks its
    // person anything: the hosting, from the bytes the application's key signed.
    #[test]
    fn a_hosting_reads_back_from_its_endorsement_once_the_signature_verifies() {
        let application = SigningKey::from_bytes(&[1; 32]);
        let registry = PublicKey::from(SigningKey::from_bytes(&[6; 32]).verifying_key());
        let hosting = Hosting { registry, name: Name::new("Acme").unwrap(), address: HostName::new("acme.example.com").unwrap() };
        let mut envelope = EnvelopeBuilder::new();
        envelope.add_must_understand(&hosting).unwrap();
        let endorsement = SignedHosting::sign(SealedEnvelope::new(envelope.encode()), &application);

        assert_eq!(endorsement.signer, PublicKey::from(application.verifying_key()));
        let opened = endorsement.open(&Limits::default(), &[Hosting::ID]).unwrap();
        assert_eq!(opened.components().read::<Hosting>().unwrap(), hosting);
    }
}
