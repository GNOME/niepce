/*
 * niepce - fwk/base/moniker.rs
 *
 * Copyright (C) 2022-2024 Hubert Figuière
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

/// URL like spec with a `scheme`:`path`.
#[derive(Debug)]
pub struct Moniker {
    scheme: String,
    path: String,
}

impl From<&String> for Moniker {
    fn from(v: &String) -> Moniker {
        Moniker::from(v.as_str())
    }
}

impl From<&str> for Moniker {
    fn from(v: &str) -> Moniker {
        let scheme;
        let path;
        if let Some(idx) = v.find(':') {
            scheme = &v[0..idx];
            path = &v[idx + 1..];
        } else {
            scheme = "";
            path = v;
        }
        Moniker {
            scheme: scheme.to_string(),
            path: path.to_string(),
        }
    }
}

impl std::fmt::Display for Moniker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.scheme.is_empty() {
            write!(f, "{}", self.path)
        } else {
            write!(f, "{}:{}", self.scheme, self.path)
        }
    }
}

impl Moniker {
    pub fn new(scheme: &str, path: &str) -> Moniker {
        Moniker {
            scheme: scheme.to_string(),
            path: path.to_string(),
        }
    }

    pub fn scheme(&self) -> &str {
        &self.scheme
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

#[cfg(test)]
mod test {

    use super::Moniker;

    #[test]
    fn test_moniker() {
        let moniker = Moniker::from("local:/tmp");

        assert_eq!(moniker.scheme(), "local");
        assert_eq!(moniker.path(), "/tmp");
        assert_eq!(&moniker.to_string(), "local:/tmp");

        let moniker = Moniker::from("/home");

        assert_eq!(moniker.scheme(), "");
        assert_eq!(moniker.path(), "/home");
        assert_eq!(&moniker.to_string(), "/home");
    }
}
