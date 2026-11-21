//! `#[derive(Minted)]`, so that a Koinon model can travel in an envelope: it converts the model's
//! struct to and from the properties of one component, keyed by the minted ids the struct declares
//! for Ankurah's `#[derive(Model)]`, the model's `#[model(id = "...")]` and each property's
//! `#[property(id = "...")]`, which the mint writes. `koinon_commons::codec` re-exports it beside
//! the `Minted` trait it implements.
//!
//! The two derives read the same attributes, and each passes over the options the other owns: this
//! one reads the ids and a field's `#[model(ephemeral)]`, and leaves `label`, `no_ffi`,
//! `renamed_from` and every other option to Ankurah's derive, which checks them. It reads an id's
//! text as Ankurah's `EntityId::from_base64` reads it, so that the two derives accept the same
//! texts.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields, Ident, LitStr, Type};

#[cfg(test)]
mod tests;

/// Implements `koinon_commons::codec::Minted` for a struct that also derives `ankurah::Model`.
#[proc_macro_derive(Minted, attributes(model, property))]
pub fn derive_minted(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(&input).unwrap_or_else(syn::Error::into_compile_error).into()
}

/// A field as an envelope carries it: a property under its minted id, or a field marked
/// `#[model(ephemeral)]`, which is neither stored nor sent and reads as its default.
enum Field<'a> {
    Property { ident: &'a Ident, ty: &'a Type, id: [u8; 32] },
    Ephemeral { ident: &'a Ident },
}

fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    let component = bytes(model_id(input)?);
    let fields = fields(input)?;
    let name = &input.ident;
    // The generated code reaches the codec through a module it keeps for this derive alone, so
    // that it names nothing the codec's users see.
    let private = quote! { ::koinon_commons::codec::__private };

    let mut writes = Vec::new();
    let mut reads = Vec::new();
    let mut checks = Vec::new();
    for field in &fields {
        match *field {
            Field::Property { ident, ty, id } => {
                let id = bytes(id);
                writes.push(quote! { properties.write(#private::MintedId::from_bytes(#id), &self.#ident)?; });
                reads.push(quote! { #ident: properties.read::<#ty>(#private::MintedId::from_bytes(#id))? });
                // Evaluated when the crate is checked, not only when it is built, so that a
                // property whose value type has no form in an envelope fails at its declaration.
                checks.push(quote_spanned! { ty.span() =>
                    const _: () = #private::check_value_type(<#ty as #private::Property>::VALUE_TYPE);
                });
            }
            Field::Ephemeral { ident } => reads.push(quote! { #ident: ::core::default::Default::default() }),
        }
    }

    Ok(quote! {
        impl #private::Minted for #name {
            const ID: #private::MintedId = #private::MintedId::from_bytes(#component);

            #[allow(unused_variables)]
            fn write_properties(
                &self,
                properties: &mut #private::PropertyWriter,
            ) -> ::core::result::Result<(), #private::EncodeError> {
                #(#writes)*
                ::core::result::Result::Ok(())
            }

            #[allow(unused_variables)]
            fn read_properties(properties: &#private::Properties<'_>) -> ::core::result::Result<Self, #private::ReadError> {
                ::core::result::Result::Ok(Self { #(#reads),* })
            }
        }
        #(#checks)*
    })
}

fn bytes(id: [u8; 32]) -> TokenStream {
    quote! { [#(#id),*] }
}

/// The struct's fields, each a property under its minted id or an ephemeral field. A field that
/// declares no id is refused, and so are two fields that declare one id.
fn fields(input: &DeriveInput) -> syn::Result<Vec<Field<'_>>> {
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(&input.generics, "a model takes no type or lifetime parameters"));
    }
    let named = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => &named.named,
            Fields::Unnamed(fields) => return Err(syn::Error::new_spanned(fields, "a model's properties are named fields")),
            Fields::Unit => return Err(syn::Error::new_spanned(&input.ident, "a model's properties are named fields, in braces")),
        },
        _ => return Err(syn::Error::new_spanned(&input.ident, "a model is a struct")),
    };
    let mut fields = Vec::new();
    for field in named {
        let ident = field.ident.as_ref().expect("a named field has a name");
        if ephemeral(field) {
            fields.push(Field::Ephemeral { ident });
            continue;
        }
        let id = property_id(field)?.ok_or_else(|| {
            syn::Error::new_spanned(ident, format!("`{ident}` declares no minted id, #[property(id = \"...\")]: run the mint"))
        })?;
        let earlier = fields.iter().find_map(|earlier| match *earlier {
            Field::Property { ident: earlier, id: earlier_id, .. } if earlier_id == id => Some(earlier),
            _ => None,
        });
        if let Some(earlier) = earlier {
            return Err(syn::Error::new_spanned(ident, format!("`{earlier}` and `{ident}` declare one property id")));
        }
        fields.push(Field::Property { ident, ty: &field.ty, id });
    }
    Ok(fields)
}

/// The model's minted id, from `#[model(id = "...")]`.
fn model_id(input: &DeriveInput) -> syn::Result<[u8; 32]> {
    let mut found = None;
    for attribute in input.attrs.iter().filter(|attribute| attribute.path().is_ident("model")) {
        attribute.parse_nested_meta(|meta| {
            if !meta.path.is_ident("id") {
                return pass_over(&meta);
            }
            if found.replace(minted_id(&meta.value()?.parse()?)?).is_some() {
                return Err(meta.error("the model declares its id twice"));
            }
            Ok(())
        })?;
    }
    found.ok_or_else(|| syn::Error::new_spanned(&input.ident, "a model declares its minted id, #[model(id = \"...\")]: run the mint"))
}

/// The property's minted id, from the field's `#[property(id = "...")]`.
fn property_id(field: &syn::Field) -> syn::Result<Option<[u8; 32]>> {
    let mut found = None;
    for attribute in field.attrs.iter().filter(|attribute| attribute.path().is_ident("property")) {
        attribute.parse_nested_meta(|meta| {
            if !meta.path.is_ident("id") {
                return pass_over(&meta);
            }
            if found.replace(minted_id(&meta.value()?.parse()?)?).is_some() {
                return Err(meta.error("the property declares its id twice"));
            }
            Ok(())
        })?;
    }
    Ok(found)
}

/// Whether the field is ephemeral as Ankurah's derive reads it: a `#[model(...)]` attribute that
/// holds the one flag `ephemeral` and nothing else.
fn ephemeral(field: &syn::Field) -> bool {
    field
        .attrs
        .iter()
        .any(|attribute| attribute.path().is_ident("model") && attribute.parse_args::<Ident>().is_ok_and(|flag| flag == "ephemeral"))
}

/// Steps over an option this derive does not read, such as `label = "..."` or `no_ffi`.
fn pass_over(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<()> {
    if meta.input.peek(syn::Token![=]) {
        meta.value()?.parse::<syn::Expr>()?;
    }
    Ok(())
}

/// The 32 bytes a minted id's text names: unpadded URL-safe base64, with no bits set past the
/// last byte.
fn minted_id(literal: &LitStr) -> syn::Result<[u8; 32]> {
    let text = literal.value();
    URL_SAFE_NO_PAD
        .decode(&text)
        .ok()
        .and_then(|bytes| <[u8; 32]>::try_from(bytes).ok())
        .ok_or_else(|| syn::Error::new_spanned(literal, format!("{text:?} is no minted id: 43 characters of unpadded URL-safe base64")))
}
