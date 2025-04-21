use std::str::FromStr;

use pyo3::{exceptions::PyValueError, PyErr};
use smol_str::SmolStr;
use snafu::Snafu;

#[derive(Debug, Clone, derive_more::Display)]
pub struct Slug(SmolStr);

#[derive(Debug, Clone, Snafu)]
#[snafu(display("expected a lowercase ASCII alphabetical character (i.e. a through z) or a digit (i.e. 0 through 9) or an underscore (i.e. _) but encountered {encountered}"))]
pub struct SlugParsingError {
    encountered: char,
}

impl From<SlugParsingError> for PyErr {
    fn from(error: SlugParsingError) -> Self {
        PyValueError::new_err(error.to_string())
    }
}

impl FromStr for Slug {
    type Err = SlugParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for c in s.chars() {
            match c {
                'a'..='z' => {}
                '0'..='9' => {}
                '_' => {}
                _ => return Err(SlugParsingError { encountered: c }),
            }
        }

        Ok(Self(s.into()))
    }
}
