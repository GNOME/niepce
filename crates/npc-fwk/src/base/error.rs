/*
 * niepce - npc-fwk/base/error.rs
 *
 * Copyright (C) 2026 Hubert Figuière
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

/// Our generic error type that can hold a context.
#[derive(Debug)]
pub struct Error {
    detail: Detail,
    context: String,
}

/// Detail: the actual error.
#[derive(Debug)]
enum Detail {
    /// Generic error
    Any(String),
    /// Other error
    Other(Box<dyn core::error::Error>),
}

/// Create an error from a string.
#[macro_export]
macro_rules! anyerror {
    ($msg:literal) => {
        $crate::Error::any($msg.into())
    };
}

impl Error {
    /// Create a Any error that contains a string.
    pub fn any(s: String) -> Self {
        Self {
            detail: Detail::Any(s),
            context: String::default(),
        }
    }

    /// Create an Other error from another error.
    pub fn with_error<E>(error: E) -> Self
    where
        E: core::error::Error + 'static,
    {
        Self {
            detail: Detail::Other(Box::new(error)),
            context: String::default(),
        }
    }

    /// Add a context to an error to create one.
    fn context<E: core::error::Error + 'static>(context: &str, error: E) -> Self {
        Self {
            detail: Detail::Other(Box::new(error)),
            context: context.into(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.detail {
            Detail::Any(s) => write!(f, "Error: {s}\n{}", self.context),
            Detail::Other(e) => write!(f, "Other error: {e}\n{}", self.context),
        }
    }
}

impl<E> From<E> for Error
where
    E: core::error::Error + 'static,
{
    fn from(error: E) -> Self {
        Self::with_error(error)
    }
}

/// This trait allow creating a result with `Error` and a context from
/// another error.
pub trait Context<T, E> {
    fn context(self, context: &str) -> Result<T, Error>;
}

impl<T, E> Context<T, E> for core::result::Result<T, E>
where
    E: core::error::Error + 'static,
{
    fn context(self, context: &str) -> core::result::Result<T, Error> {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => Err(Error::context(context, err)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_error_creation() {
        let result: std::io::Result<()> = Err(std::io::Error::other("oh no!"));

        let result2 = result.context("Test");
        assert_eq!(result2.err().unwrap().context, "Test");

        let error = anyerror!("yeah!");
        assert!(matches!(error.detail, Detail::Any(x) if x == "yeah!"));
        assert_eq!(error.context, String::default());

        let error = std::io::Error::other("oh no!");
        let error2 = Error::from(error);
        assert!(matches!(error2.detail, Detail::Other(_)));
        assert_eq!(error2.context, String::default());
    }
}
