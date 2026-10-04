use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! string_id {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        pub struct $name(pub String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

string_id!(
    FeatureId,
    "Globally unique canonical feature id. Overture-style uuid-based ids are accepted verbatim."
);
string_id!(SourceId, "Identifier of the upstream source system (orbis, overture, osm, cv, customer).");
string_id!(MapVersion, "Monotonic version label of a map release artifact.");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_hashable_and_comparable() {
        let a = FeatureId::new("0x1234");
        let b = FeatureId::new("0x1234");
        assert_eq!(a, b);
        let mut set = std::collections::HashSet::new();
        set.insert(a);
        assert!(set.contains(&b));
    }
}
