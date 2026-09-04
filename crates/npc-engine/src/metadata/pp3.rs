/*
 * niepce - engine/metadata/pp3.rs
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

//! import PP3 files from RawTherapee

use std::collections::HashMap;

use configparser::ini::Ini;

use npc_fwk::on_err_out;

/// XMP namespace for RT compatibility
const NS_RT_COMPAT: &str = "http://xmlns.figuiere.net/niepce/rt-compat";
const NS_RT_COMPAT_PREFIX: &str = "pp3";

type Pp3Map = HashMap<String, HashMap<String, Option<String>>>;

/// Load a PP3 file into `Ppp3Map`.
pub fn load_pp3<P>(file: P) -> Result<Pp3Map, String>
where
    P: AsRef<std::path::Path>,
{
    let mut config = Ini::new_cs();
    config.load(file)
}

/// Convert the string to a valid xpath string.
///
/// Currently this remove spaces and `&`.
fn to_valid_xpath(input: &str) -> String {
    input
        .chars()
        .filter(|c| *c != ' ' && *c != '&')
        .collect::<String>()
}

/// Append the PP3 to an XMP.
pub fn append_to_xmp(
    from: &Pp3Map,
    xmp: &mut exempi2::Xmp,
    base_prop: Option<(&str, &str)>,
) -> exempi2::Result<()> {
    use exempi2::PropFlags;

    let prefix = match exempi2::namespace_prefix(NS_RT_COMPAT) {
        Ok(prefix) => prefix.to_string(),
        Err(_) => exempi2::register_namespace(NS_RT_COMPAT, NS_RT_COMPAT_PREFIX)
            .expect("Failed to register XMP namespace")
            .to_string(),
    };

    from.iter().for_each(|(group, value)| {
        let group = to_valid_xpath(group);
        value.iter().for_each(|(key, value)| {
            if let Some(value) = value {
                let xpath;
                let ns;
                if let Some(base_prop) = base_prop {
                    // XXX can we simplify this?
                    xpath = format!("{}/{prefix}{group}/{prefix}{key}", base_prop.1);
                    ns = base_prop.0;
                } else {
                    xpath = format!("{group}/{prefix}{key}");
                    ns = NS_RT_COMPAT;
                }
                on_err_out!(xmp.set_property(ns, &xpath, value, PropFlags::NONE));
            }
        })
    });

    Ok(())
}

#[cfg(test)]
mod test {
    use exempi2;

    use super::{NS_RT_COMPAT, NS_RT_COMPAT_PREFIX, append_to_xmp, to_valid_xpath};
    use crate::metadata::xmp::{self, NIEPCE_XMP_NAMESPACE};
    use npc_fwk::utils::xmp::NsDef;

    #[test]
    fn test_xpath_normalise() {
        let xpath = to_valid_xpath("Some Value");
        assert_eq!(xpath, "SomeValue");
        let xpath = to_valid_xpath("Peanut&Jelly");
        assert_eq!(xpath, "PeanutJelly");
    }

    #[test]
    fn test_load() {
        let _manager = xmp::xmp_manager(Some(vec![NsDef {
            ns: NS_RT_COMPAT,
            prefix: NS_RT_COMPAT_PREFIX,
        }]));
        let map = super::load_pp3("tests/test.pp3");
        assert!(map.is_ok());

        let map = map.unwrap();
        println!("map {map:?}");
        assert!(map.contains_key("Version"));

        let mut xmp = exempi2::Xmp::new();
        append_to_xmp(&map, &mut xmp, None).expect("Couldn't append to XMP");
        assert!(xmp.has_property(NS_RT_COMPAT, "Version"));

        let mut xmp = exempi2::Xmp::new();
        append_to_xmp(&map, &mut xmp, Some((NIEPCE_XMP_NAMESPACE, "PP3ForJPEG")))
            .expect("Couldn't append to XMP");
        assert!(xmp.has_property(NIEPCE_XMP_NAMESPACE, "PP3ForJPEG/pp3:Version"));
    }
}
