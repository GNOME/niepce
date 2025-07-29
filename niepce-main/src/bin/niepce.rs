/*
 * niepce - bin/niepce.rs
 *
 * Copyright (C) 2023 Hubert Figuière
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

use gettextrs::*;

use niepce_core::{NiepceApplication, config};
use npc_fwk::{ExempiManager, dbg_out};

fn main() {
    bindtextdomain(config::GETTEXT_PACKAGE, config::LOCALEDIR).expect("bindtextdomain failed");
    bind_textdomain_codeset(config::GETTEXT_PACKAGE, "UTF-8")
        .expect("bind textdomain codeset failed");
    textdomain(config::GETTEXT_PACKAGE).expect("textdomain failed");

    niepce_core::niepce_init();

    let _ = ExempiManager::new(None);

    dbg_out!("Starting up. DEBUG is on");

    let app = NiepceApplication::new();
    app.main();
}
