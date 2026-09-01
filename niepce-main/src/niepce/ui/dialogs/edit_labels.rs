/*
 * niepce - niepce/ui/dialogs/edit_labels.rs
 *
 * Copyright (C) 2024-2026 Hubert Figuière
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

use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::Arc;

use adw::prelude::*;
use npc_fwk::{adw, glib, gtk4};

use npc_engine::catalog;
use npc_engine::library::actions;
use npc_engine::libraryclient::{LibraryClient, LibraryClientHost};
use npc_fwk::toolkit::{Controller, ControllerImplCell, DialogController, UiController};
use npc_fwk::{controller_imp_imp, send_async_local};

use crate::NiepceApplication;

const NUM_LABELS: usize = 5;

pub enum InMsg {
    ColourChanged(usize),
    NameChanged(usize),
    CloseRequest,
}

pub struct EditLabels {
    imp_: ControllerImplCell<InMsg, ()>,
    client: Arc<LibraryClient>,
    app: Weak<NiepceApplication>,
    labels: Vec<catalog::Label>,
    colours: Vec<gtk4::ColorDialogButton>,
    entries: Vec<gtk4::Entry>,
    status: RefCell<[bool; NUM_LABELS]>,
    dialog: adw::Window,
}

impl Controller for EditLabels {
    type InMsg = InMsg;
    type OutMsg = ();

    controller_imp_imp!(imp_);

    fn dispatch(&self, msg: InMsg) {
        match msg {
            InMsg::ColourChanged(idx) | InMsg::NameChanged(idx) => self.changed(idx),
            InMsg::CloseRequest => {
                self.update_labels();
                self.close();
            }
        }
    }
}

impl UiController for EditLabels {
    fn widget(&self) -> &gtk4::Widget {
        self.dialog.upcast_ref()
    }
}

impl DialogController for EditLabels {
    fn dialog(&self) -> &adw::Window {
        &self.dialog
    }
}

impl EditLabels {
    pub fn new(client: &Rc<LibraryClientHost>, app: Weak<NiepceApplication>) -> Rc<EditLabels> {
        let builder = gtk4::Builder::from_resource("/net/figuiere/Niepce/ui/edit_labels.ui");
        let provider = client.ui_provider();
        let mut labels = vec![];

        assert!(provider.label_count() >= NUM_LABELS);

        for idx in 0..NUM_LABELS {
            labels.push(provider.label_at(idx));
        }
        get_widget!(builder, adw::Window, edit_labels);

        let mut ctrl = EditLabels {
            imp_: ControllerImplCell::default(),
            client: client.client().clone(),
            app,
            labels,
            entries: vec![],
            colours: vec![],
            status: RefCell::new([false; 5]),
            dialog: edit_labels,
        };

        ctrl.build_widget(builder);

        let ctrl = Rc::new(ctrl);
        <Self as DialogController>::start(&ctrl);

        ctrl
    }

    fn build_widget(&mut self, builder: gtk4::Builder) {
        let colour_dialog = gtk4::ColorDialog::new();
        for idx in 0..NUM_LABELS {
            self.colours.push(
                builder
                    .object::<gtk4::ColorDialogButton>(format!("colourbutton{}", idx + 1))
                    .unwrap(),
            );
            self.entries.push(
                builder
                    .object::<gtk4::Entry>(format!("value{}", idx + 1))
                    .unwrap(),
            );

            let colour = self.labels[idx].colour();
            self.colours[idx].set_dialog(&colour_dialog);
            self.colours[idx].set_rgba(&(colour.clone()).into());
            self.entries[idx].set_text(self.labels[idx].label());

            let sender = self.sender();
            self.colours[idx].connect_notify(Some("rgba"), move |_, _| {
                send_async_local!(InMsg::ColourChanged(idx), sender);
            });
            let sender = self.sender();
            self.entries[idx].connect_changed(move |_| {
                send_async_local!(InMsg::NameChanged(idx), sender);
            });
        }

        let sender = self.sender();
        self.dialog.connect_close_request(move |_| {
            send_async_local!(InMsg::CloseRequest, sender);
            glib::Propagation::Proceed
        });
    }

    fn changed(&self, idx: usize) {
        self.status.borrow_mut()[idx] = true;
    }

    fn update_labels(&self) {
        let statuses = self.status.borrow();
        let mut labels = vec![];
        let mut old = vec![];
        let mut new = vec![];
        for status in statuses.iter().enumerate() {
            if !status.1 {
                continue;
            }
            new.push((
                self.entries[status.0].text().to_string(),
                self.colours[status.0].rgba().into(),
            ));
            old.push((
                self.labels[status.0].label().to_string(),
                self.colours[status.0].rgba().into(),
            ));
            labels.push(self.labels[status.0].id());
        }
        actions::update_labels(&self.client, &self.app, &labels, old, new)
    }
}
