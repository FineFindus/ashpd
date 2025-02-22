use serde::{Deserialize, Serialize};
use zbus::zvariant::Type;

/// The application ID.
///
/// See <https://developer.gnome.org/documentation/tutorials/application-id.html>.
#[derive(Debug, Serialize, Deserialize, Type, PartialEq, Eq, Hash, Clone)]
pub struct AppID(String);

impl TryFrom<String> for AppID {
    type Error = crate::Error;

    fn try_from(string: String) -> Result<Self, Self::Error> {
        if is_valid_app_id(&string) {
            Ok(Self(string))
        } else {
            Err(Self::Error::InvalidAppID)
        }
    }
}

impl TryFrom<&str> for AppID {
    type Error = crate::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        AppID::try_from(s.to_string())
    }
}

impl From<AppID> for String {
    fn from(value: AppID) -> String {
        value.0
    }
}

impl AsRef<str> for AppID {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl std::fmt::Display for AppID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

/// The ID of a file in the document store.
#[derive(Debug, Serialize, Deserialize, Type, PartialEq, Eq, Hash, Clone)]
pub struct DocumentID(String);

impl From<&str> for DocumentID {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl From<String> for DocumentID {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<DocumentID> for String {
    fn from(value: DocumentID) -> String {
        value.0
    }
}

impl AsRef<str> for DocumentID {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl std::fmt::Display for DocumentID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

// Helpers

fn is_valid_app_id(string: &str) -> bool {
    let len = string.len();

    // The app id has to be between 0 < len <= 255
    if len == 0 || 255 < len {
        return false;
    }

    let elements: Vec<&str> = string.split('.').collect();
    let segments = elements.len();

    if segments < 2 {
        return false;
    }

    for (idx_segment, element) in elements.iter().enumerate() {
        // No empty segments.
        if element.is_empty() {
            return false;
        }

        for (idx_char, c) in element.chars().enumerate() {
            // First char cannot be a digit.
            if idx_char == 0 && c.is_ascii_digit() {
                return false;
            }
            if !is_valid_app_id_char(c) {
                return false;
            }
            // Only the last segment can contain `-`.
            if idx_segment < segments - 1 && c == '-' {
                return false;
            }
        }
    }

    true
}

/// Only valid chars are a-z A-Z 0-9 - _
fn is_valid_app_id_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '-' | '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_app_id() {
        assert!(is_valid_app_id("a.b"));
        assert!(is_valid_app_id("a_c.b_c.h_c"));
        assert!(is_valid_app_id("a.c-b"));
        assert!(is_valid_app_id("a.c2.d"));

        assert!(!is_valid_app_id("a"));
        assert!(!is_valid_app_id(""));
        assert!(!is_valid_app_id("a-z.b.c.d"));
        assert!(!is_valid_app_id("a.b-z.c.d"));
        assert!(!is_valid_app_id("a.b.c-z.d"));
        assert!(!is_valid_app_id("a.0b.c"));
        assert!(!is_valid_app_id("a..c"));
        assert!(!is_valid_app_id("a.é"));
        assert!(!is_valid_app_id("a.京"));

        // Tests from
        // https://github.com/bilelmoussaoui/flatpak-vscode/blob/master/src/test/suite/extension.test.ts

        assert!(is_valid_app_id("_org.SomeApp"));
        assert!(is_valid_app_id("com.org.SomeApp"));
        assert!(is_valid_app_id("com.org_._SomeApp"));
        assert!(is_valid_app_id("com.org._1SomeApp"));
        assert!(is_valid_app_id("com.org._1_SomeApp"));
        assert!(is_valid_app_id("VeryLongApplicationId.VeryLongApplicationId.VeryLongApplicationId.VeryLongApplicationId.VeryLongApplicationId.VeryLongApplicationId.VeryLongApplicationId.VeryLongApplicationId.VeryLongApplicationId.VeryLongApplicationId.VeryLongApplicationId.a111111111111"));

        assert!(!is_valid_app_id("com.org-._SomeApp"));
        assert!(!is_valid_app_id("package"));
        assert!(!is_valid_app_id("NoDot"));
        assert!(!is_valid_app_id("No-dot"));
        assert!(!is_valid_app_id("No_dot"));
        assert!(!is_valid_app_id("Has.Two..Consecutive.Dots"));
        assert!(!is_valid_app_id("HasThree...Consecutive.Dots"));
        assert!(!is_valid_app_id(".StartsWith.A.Period"));
        assert!(!is_valid_app_id("."));
        assert!(!is_valid_app_id("Ends.With.A.Period."));
        assert!(!is_valid_app_id("0P.Starts.With.A.Digit"));
        assert!(!is_valid_app_id("com.org.1SomeApp"));
        assert!(!is_valid_app_id("Element.Starts.With.A.1Digit"));
        assert!(!is_valid_app_id("VeryLongApplicationId.VeryLongApplicationId.VeryLongApplicationId.VeryLongApplicationId.VeryLongApplicationId.VeryLongApplicationId.VeryLongApplicationId.VeryLongApplicationId.VeryLongApplicationId.VeryLongApplicationId.VeryLongApplicationId.a1111111111112"));
        assert!(!is_valid_app_id(""));
        assert!(!is_valid_app_id("contains.;nvalid.characters"));
        assert!(!is_valid_app_id("con\nins.invalid.characters"));
        assert!(!is_valid_app_id("con/ains.invalid.characters"));
        assert!(!is_valid_app_id("conta|ns.invalid.characters"));
        assert!(!is_valid_app_id("contæins.inva_å_lid.characters"));
    }
}
