/*
 * niepce - engine/metadata.rs
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

//! Metadata handling in engine. This is all the application specific.

pub mod analog_exif;
pub(crate) mod pp3;
pub mod xmp;

use std::path::Path;

use exempi2::{PropFlags, Xmp};

use crate::catalog::filebundle::FileBundle;
use crate::catalog::libfile::FileType;
use npc_fwk::{Date, DateExt};
use npc_fwk::{dbg_out, on_err_out};
use xmp::NIEPCE_XMP_NAMESPACE;

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    /// Error from the PP3 loading.
    #[error("PP3 load error: {0}")]
    Pp3Load(String),
    /// XMP Error
    #[error("XMP error: {0}")]
    Exempi(#[from] exempi2::Error),
    /// If the bundle does have the image that the file refer to.
    #[error("Associated file not found.")]
    NotFound,
}

/// Check if we need to merge PP3
///
/// If the PP3 data already exist, then we don't need to merge
/// in that case we try to update the sync status.
fn need_merge_pp3(xmp: &mut Xmp, sidecar: &Path, base: &str) -> bool {
    if xmp.has_property(NIEPCE_XMP_NAMESPACE, base) {
        dbg_out!("Found {base}");
        let mut flags = PropFlags::default();
        if let Ok(stored_digest) = xmp.get_property(
            NIEPCE_XMP_NAMESPACE,
            &format!("{base}/niepce:{}", xmp::PP3_DIGEST),
            &mut flags,
        ) && let Ok(digest) = npc_fwk::utils::md5_digest(sidecar)
            && stored_digest.to_string() != digest
        {
            // There is no other indication of modification than the file
            // modification date, which can be incorrect.
            if let Ok(sidecar_date) = npc_fwk::utils::modification_date(sidecar)
                && let Ok(xmp_date) = xmp
                    .get_property_date(
                        NIEPCE_XMP_NAMESPACE,
                        &format!("{base}/niepce:{}", xmp::PP3_MODIFIED),
                        &mut flags,
                    )
                    .or_else(|_| {
                        xmp.get_property_date(
                            NIEPCE_XMP_NAMESPACE,
                            &format!("{base}/niepce:{}", xmp::PP3_IMPORTED),
                            &mut flags,
                        )
                    })
                    .map(|date| Date::from_exempi(&date))
            {
                // Since the digest differ, if the mod date didn't change
                // we assume it was changed after import.
                let status = if sidecar_date >= xmp_date {
                    -1 // XMP is older
                } else {
                    1 // XMP is more recent
                };
                on_err_out!(xmp.set_property_i32(
                    NIEPCE_XMP_NAMESPACE,
                    &format!("{base}/niepce:{}", xmp::PP3_SYNC_STATUS),
                    status,
                    PropFlags::NONE
                ));
            }
        }
        false
    } else {
        true
    }
}

/// Determine the property for that pp3 sidecar file.
fn property_for_bundle_type(bundle: &FileBundle, sidecar: &Path) -> Option<&'static str> {
    match bundle.bundle_type() {
        FileType::Raw | FileType::Image => {
            if bundle.main().with_added_extension("pp3") == sidecar {
                if bundle.bundle_type() == FileType::Raw {
                    Some(xmp::PP3_FOR_RAW)
                } else {
                    Some(xmp::PP3_FOR_JPEG)
                }
            } else {
                None
            }
        }
        FileType::RawJpeg => {
            if bundle.main().with_added_extension("pp3") == sidecar {
                Some(xmp::PP3_FOR_RAW)
            } else if bundle.jpeg().with_added_extension("pp3") == sidecar {
                Some(xmp::PP3_FOR_JPEG)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Merge the PP3 data from the `bundle` into the the `xmp`.
/// Return Ok(true) is the merge happened.
pub(crate) fn merge_pp3_from_bundle(
    bundle: &FileBundle,
    sidecar: &Path,
    xmp: &mut Xmp,
) -> Result<bool, Error> {
    let base = property_for_bundle_type(bundle, sidecar);

    if let Some(base) = base {
        if !need_merge_pp3(xmp, sidecar, base) {
            dbg_out!("need merge pp3 return false -> no merge");
            return Ok(false);
        }
        dbg_out!("need merge pp3 return true -> merge");
    }

    let pp3 = pp3::load_pp3(sidecar).map_err(Error::Pp3Load)?;
    if let Some(base) = base {
        if let Ok(digest) = npc_fwk::utils::md5_digest(sidecar) {
            on_err_out!(xmp.set_property(
                NIEPCE_XMP_NAMESPACE,
                &format!("{base}/niepce:{}", xmp::PP3_DIGEST),
                &digest,
                PropFlags::NONE
            ));
        }

        let imported = npc_fwk::Date::now().into_xmpdate();
        on_err_out!(xmp.set_property_date(
            NIEPCE_XMP_NAMESPACE,
            &format!("{base}/niepce:{}", xmp::PP3_IMPORTED),
            &imported,
            PropFlags::NONE
        ));
        on_err_out!(xmp.set_property_date(
            NIEPCE_XMP_NAMESPACE,
            &format!("{base}/niepce:{}", xmp::PP3_MODIFIED),
            &imported,
            PropFlags::NONE
        ));

        pp3::append_to_xmp(&pp3, xmp, Some((NIEPCE_XMP_NAMESPACE, base)))?;
        if let Some(imported_from) = sidecar.file_name().and_then(std::ffi::OsStr::to_str) {
            on_err_out!(xmp.set_property(
                NIEPCE_XMP_NAMESPACE,
                &format!("{base}/niepce:{}", xmp::PP3_FROM),
                imported_from,
                PropFlags::NONE
            ));
        }
        on_err_out!(xmp.set_property(
            NIEPCE_XMP_NAMESPACE,
            &format!("{base}/niepce:{}", xmp::PP3_SYNC_STATUS),
            "0",
            PropFlags::NONE
        ));
        Ok(true)
    } else {
        Err(Error::NotFound)
    }
}
