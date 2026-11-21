//! How the derive reads a struct: the ids it takes, and the declarations it refuses. What the code
//! it generates does is tested in the codec, on models.

use super::*;

const DEVICE: &str = "ULVl4nlCm9s4wkRAosvq9iDNwy5ePclWoGHd_c60z-g";
const STATION: &str = "oonHtZVf7XkOnRruznuVrlnOlYVg0oRRgopPdDFYTEM";
const NAME: &str = "jkm85rInO7Xjr4OOVI9oYb2vgey49mlm7k3jFJFVVq8";

fn parsed(tokens: TokenStream) -> DeriveInput {
    syn::parse2(tokens).unwrap()
}

fn decoded(text: &str) -> [u8; 32] {
    URL_SAFE_NO_PAD.decode(text).unwrap().try_into().unwrap()
}

/// Each field's property id, or `None` for an ephemeral field.
fn ids(input: &DeriveInput) -> Vec<Option<[u8; 32]>> {
    let fields = fields(input).unwrap();
    fields
        .iter()
        .map(|field| match *field {
            Field::Property { id, .. } => Some(id),
            Field::Ephemeral { .. } => None,
        })
        .collect()
}

fn refusal(result: syn::Result<impl Sized>) -> String {
    result.err().expect("the declaration is refused").to_string()
}

// The Koinon Commons' Device as the mint wrote it, with Ankurah's other options beside the ids.
#[test]
fn the_ids_are_the_bytes_the_attributes_name_and_ankurahs_options_pass_through() {
    let input = parsed(quote! {
        #[model(label = "devices.device")]
        #[model(id = "ULVl4nlCm9s4wkRAosvq9iDNwy5ePclWoGHd_c60z-g")]
        #[model(no_ffi)]
        pub struct Device {
            #[property(id = "oonHtZVf7XkOnRruznuVrlnOlYVg0oRRgopPdDFYTEM")]
            pub station: Ref<Station>,
            #[active_type(LWW)]
            #[property(renamed_from = "label", id = "jkm85rInO7Xjr4OOVI9oYb2vgey49mlm7k3jFJFVVq8")]
            pub name: String,
            #[model(ephemeral)]
            pub shown: bool,
        }
    });
    assert_eq!(model_id(&input).unwrap(), decoded(DEVICE));
    assert_eq!(ids(&input), [Some(decoded(STATION)), Some(decoded(NAME)), None]);
}

#[test]
fn a_struct_without_its_model_id_is_refused() {
    let input = parsed(quote! { #[model(label = "devices.device")] pub struct Device { #[property(id = #STATION)] station: String } });
    assert!(refusal(model_id(&input)).contains("run the mint"));
}

#[test]
fn a_field_without_its_property_id_is_refused_by_name() {
    let input = parsed(quote! { pub struct Device { #[property(id = #STATION)] station: String, #[active_type(LWW)] name: String } });
    let refused = refusal(fields(&input));
    assert!(refused.contains("`name` declares no minted id"), "{refused}");
}

#[test]
fn two_fields_with_one_property_id_are_refused_naming_both() {
    let input = parsed(quote! { pub struct Device { #[property(id = #NAME)] name: String, #[property(id = #NAME)] label: String } });
    let refused = refusal(fields(&input));
    assert!(refused.contains("`name` and `label` declare one property id"), "{refused}");
}

// Ankurah's derive refuses a model's id declared twice, and takes the last of a property's.
#[test]
fn an_id_declared_twice_is_refused() {
    let input = parsed(quote! { #[model(id = #DEVICE)] #[model(id = #DEVICE)] pub struct Device {} });
    assert!(refusal(model_id(&input)).contains("declares its id twice"));
    let input = parsed(quote! { pub struct Device { #[property(id = #NAME)] #[property(id = #STATION)] name: String } });
    assert!(refusal(fields(&input)).contains("declares its id twice"));
}

// The texts Ankurah's derive refuses: too short, padded, outside the alphabet, and with bits set
// past the last byte.
#[test]
fn a_text_that_is_no_minted_id_is_refused() {
    for text in [
        "AAAAAAAAAAAAAAAAAAAAAA",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
        "******UGBwgJCgsMDQ4PEAAAAAAAAAAAAAAAAAAAAAA",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAB",
    ] {
        let input = parsed(quote! { #[model(id = #text)] pub struct Device {} });
        assert!(refusal(model_id(&input)).contains("is no minted id"), "{text}");
    }
    let input = parsed(quote! { #[model(id = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")] pub struct Device {} });
    assert_eq!(model_id(&input).unwrap(), [0; 32]);
}

#[test]
fn a_tuple_struct_a_unit_struct_an_enum_and_a_generic_struct_are_refused() {
    for tokens in [
        quote! { #[model(id = #DEVICE)] pub struct Device(String); },
        quote! { #[model(id = #DEVICE)] pub struct Device; },
        quote! { #[model(id = #DEVICE)] pub enum Device { Printer } },
        quote! { #[model(id = #DEVICE)] pub struct Device<T> { #[property(id = #NAME)] name: T } },
    ] {
        assert!(fields(&parsed(tokens.clone())).is_err(), "{tokens}");
    }
}

// Ankurah's derive takes a field as ephemeral only when its #[model(...)] holds the one flag, so
// a field with more in it is a property, which this derive gives its id.
#[test]
fn a_field_is_ephemeral_exactly_when_ankurahs_derive_reads_it_so() {
    let input = parsed(quote! {
        pub struct Device {
            #[model(ephemeral)]
            shown: bool,
            #[model(ephemeral, other)]
            #[property(id = #NAME)]
            name: String,
        }
    });
    assert_eq!(ids(&input), [None, Some(decoded(NAME))]);
}
