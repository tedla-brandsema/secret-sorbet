/// A name/value pair whose exact UTF-8 bytes and position enter the context.
///
/// No trimming, Unicode normalization, sorting, or case folding is performed.
/// Derivation rejects empty names or values. Both fields may contain sensitive
/// account metadata; their derived debug representation is not safe for logs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable {
    pub name: String,
    pub value: String,
}

impl Variable {
    /// Store exact strings without validation or canonicalization.
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}
