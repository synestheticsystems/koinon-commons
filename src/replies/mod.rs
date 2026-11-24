//! What any response may say of its request, whatever was asked: that it was refused and why, and
//! which components the receiver did not understand. Every response holds the Ankurah Commons'
//! `Response`, which names the request it answers. It is a refusal when it holds `Refusal`;
//! otherwise it is the answer, which holds the components its request names, or `Response` alone
//! when the request asks for nothing back. An answer's facts, such as a scale's reading, are
//! components of the namespaces their meaning puts them in.
// OPEN: the namespace's name, which every label here holds, and so every minted id, for good:
// replies, a word for an answer or a refusal alike, rather than responses or dispositions. Lean:
// replies.

mod not_understood;
mod refusal;

pub use not_understood::*;
pub use refusal::*;
