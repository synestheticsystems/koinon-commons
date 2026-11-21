//! Lists of at least one item and none twice, such as the stations at which an authority is
//! honoured: each kept in the order given, and stored and carried as a JSON array of its items'
//! texts, which reads back only as a list that keeps those rules.

use std::{collections::HashSet, hash::Hash};

/// An item of a list that [`distinct_list!`] declares: the one text it is written as in the
/// list's JSON array, and how such a text reads back.
pub(crate) trait ListedText: Sized + Clone + Eq + Hash {
    fn listed_text(&self) -> String;

    /// The item that `text` writes, or why it writes none.
    fn from_listed_text(text: &str) -> Result<Self, String>;
}

/// The first of `items` that repeats one before it, found in one pass with a set of those seen,
/// so that checking a long list a peer sent costs no more than reading it.
pub(crate) fn first_repeat<T: Eq + Hash>(items: &[T]) -> Option<&T> {
    let mut seen = HashSet::with_capacity(items.len());
    items.iter().find(|item| !seen.insert(*item))
}

/// Declares `$list`, a list of `$item`s, at least one and none twice, in the order given, with
/// `$items` giving them; `$refused`, why items are no `$list`, whose words name one item as
/// `$noun`; and the list's `Property`, a JSON array of the items' [`ListedText`], which reads back
/// only as a list that keeps these rules: an item of another shape, an empty list or an item
/// listed twice is a value the list refuses, not JSON that does not parse.
macro_rules! distinct_list {
    (
        $(#[$list_doc:meta])*
        pub struct $list:ident(Vec<$item:ty>), each a $noun:literal, given by $items:ident;

        $(#[$refused_doc:meta])*
        pub enum $refused:ident;
    ) => {
        $(#[$list_doc])*
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $list(Vec<$item>);

        impl $list {
            pub fn new(items: Vec<$item>) -> Result<Self, $refused> {
                if items.is_empty() {
                    return Err($refused::Empty);
                }
                if let Some(repeated) = $crate::stored_forms::first_repeat(&items) {
                    return Err($refused::Twice(repeated.clone()));
                }
                Ok($list(items))
            }

            pub fn $items(&self) -> &[$item] {
                &self.0
            }

            pub fn contains(&self, item: &$item) -> bool {
                self.0.contains(item)
            }
        }

        $(#[$refused_doc])*
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum $refused {
            /// The list names none.
            Empty,
            /// This is listed twice: the first that repeats one listed before it.
            Twice($item),
        }

        impl ::std::fmt::Display for $refused {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    $refused::Empty => write!(f, concat!("no ", $noun, " is listed")),
                    $refused::Twice(item) => {
                        write!(f, concat!("the ", $noun, " {} is listed twice"), $crate::stored_forms::ListedText::listed_text(item))
                    }
                }
            }
        }

        impl ::std::error::Error for $refused {}

        impl ::ankurah::Property for $list {
            const VALUE_TYPE: &'static str = "json";

            fn into_value(&self) -> Result<Option<::ankurah::value::Value>, ::ankurah::property::PropertyError> {
                let texts = self.0.iter().map(|item| ::serde_json::Value::String($crate::stored_forms::ListedText::listed_text(item)));
                Ok(Some(::ankurah::value::Value::Json(texts.collect())))
            }

            fn from_value(value: Option<::ankurah::value::Value>) -> Result<Self, ::ankurah::property::PropertyError> {
                use ::ankurah::{property::PropertyError, value::Value};
                let refused = |value: String| PropertyError::InvalidValue { value, ty: stringify!($list).to_owned() };
                match value {
                    Some(json @ Value::Json(_)) => {
                        let texts: Vec<String> = json.parse_as_json().map_err(|error| PropertyError::DeserializeError(Box::new(error)))?;
                        let items = texts
                            .iter()
                            .map(|text| <$item as $crate::stored_forms::ListedText>::from_listed_text(text).map_err(refused))
                            .collect::<Result<_, _>>()?;
                        $list::new(items).map_err(|error| refused(error.to_string()))
                    }
                    Some(other) => Err(PropertyError::InvalidVariant { given: other, ty: stringify!($list).to_owned() }),
                    None => Err(PropertyError::Missing),
                }
            }
        }
    };
}

pub(crate) use distinct_list;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_item_to_repeat_one_before_it_is_found() {
        assert_eq!(first_repeat(&[3, 1, 4, 1, 5, 3]), Some(&1));
        assert_eq!(first_repeat(&[2, 7, 1, 8]), None);
        assert_eq!(first_repeat::<u8>(&[]), None);
    }
}
