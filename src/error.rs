use std::fmt;

/// Errors from reading or writing a WARC archive.
///
/// None carries a C2PA validation status code — see [`Error::code`].
#[derive(Debug)]
pub enum Error {
    /// No C2PA manifest record was present in the archive.
    NotFound,
    /// A WARC record was structurally malformed.
    InvalidRecord(String),
    /// An underlying I/O failure.
    Io(std::io::Error),
}

impl Error {
    /// The registered C2PA validation status code for this error, or `None`
    /// when the condition carries no status code.
    ///
    /// Always `None` here. [`Error::NotFound`] means the archive carries no
    /// provenance, which is not a failure; the other two are archive-parsing
    /// and I/O problems that occur before any manifest is located. The
    /// specification defines no WARC-specific codes.
    ///
    /// Every crate in this family exposes this method, so a dispatcher handling
    /// several embedding methods can ask the same question of any of them.
    pub fn code(&self) -> Option<&'static str> {
        None
    }

    /// Whether this error means the archive carries no provenance at all, as
    /// opposed to provenance that was found and rejected.
    pub fn is_no_manifest_located(&self) -> bool {
        matches!(self, Self::NotFound)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "no C2PA manifest record found"),
            Self::InvalidRecord(s) => write!(f, "invalid WARC record: {s}"),
            Self::Io(e) => write!(f, "I/O error: {e}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The specification defines no WARC-specific codes, so none may appear.
    /// Guards against a later edit inventing one.
    #[test]
    fn no_variant_claims_a_status_code() {
        for e in [
            Error::NotFound,
            Error::InvalidRecord("x".into()),
            Error::Io(std::io::Error::other("x")),
        ] {
            assert_eq!(e.code(), None, "{e:?} claimed a status code");
        }
    }

    /// An archive carrying no manifest is unsigned; a malformed one is not.
    /// Those two must stay distinguishable to a caller reporting provenance.
    #[test]
    fn only_a_missing_manifest_means_unsigned() {
        assert!(Error::NotFound.is_no_manifest_located());
        assert!(!Error::InvalidRecord("x".into()).is_no_manifest_located());
        assert!(!Error::Io(std::io::Error::other("x")).is_no_manifest_located());
    }
}
