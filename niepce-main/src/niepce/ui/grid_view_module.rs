/*
 * niepce - niepce/ui/grid_view_module.rs
 *
 * Copyright (C) 2022-2025 Hubert Figuière
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

use std::rc::Rc;
use std::sync::Arc;

use gtk4::prelude::*;
use npc_fwk::{gdk4, gio, glib, gtk4};

use npc_engine::catalog;
use npc_engine::library::notification::LibNotification;
use npc_engine::libraryclient::{ClientInterface, LibraryClient, LibraryClientHost};
use npc_fwk::toolkit::widgets::Dock;
use npc_fwk::toolkit::widgets::MetadataPropertyBag;
use npc_fwk::toolkit::{Controller, ControllerImplCell, UiController};
use npc_fwk::{dbg_out, send_async_local};

use crate::niepce::ui::metadata_pane_controller::MetadataOutputMsg;
use crate::niepce::ui::{
    ImageGridView, LibraryModule, MetadataPaneController, SelectionController,
};

pub enum GridMsg {
    Click(gtk4::GestureClick, f64, f64),
    ChangeRating(catalog::LibraryId, i32),
    MetadataChanged(MetadataPropertyBag, MetadataPropertyBag),
}

pub struct GridViewModule {
    imp_: ControllerImplCell<GridMsg, ()>,
    selection_controller: Rc<SelectionController>,
    pub image_grid_view: ImageGridView,
    metadatapanecontroller: Rc<MetadataPaneController>,
    context_menu: gtk4::PopoverMenu,
    widget: gtk4::Paned,
}

impl Controller for GridViewModule {
    type InMsg = GridMsg;
    type OutMsg = ();

    npc_fwk::controller_imp_imp!(imp_);

    fn dispatch(&self, msg: GridMsg) {
        match msg {
            GridMsg::Click(gesture, x, y) => self.on_librarylistview_click(&gesture, x, y),
            GridMsg::ChangeRating(id, rating) => {
                self.selection_controller.set_rating_of(id, rating)
            }
            GridMsg::MetadataChanged(new, old) => {
                self.selection_controller.set_properties(&new, &old)
            }
        }
    }
}

impl UiController for GridViewModule {
    fn widget(&self) -> &gtk4::Widget {
        // In this the assumption is that widget has been set at
        // construction time from the C++ impl and since the only way
        // to do so is by calling the new associated function, it
        // should be ok.
        self.widget.upcast_ref()
    }

    fn actions(&self) -> Option<(&str, &gio::ActionGroup)> {
        None
    }
}

impl LibraryModule for GridViewModule {
    fn widget(&self) -> &gtk4::Widget {
        UiController::widget(self)
    }
}

impl GridViewModule {
    pub fn new(
        selection_controller: &Rc<SelectionController>,
        menu: &gio::Menu,
        libclient_host: &LibraryClientHost,
    ) -> Rc<Self> {
        let widget = gtk4::Paned::new(gtk4::Orientation::Horizontal);
        let context_menu = gtk4::PopoverMenu::from_model(Some(menu));
        let model = selection_controller.list_store().selection_model();
        let image_grid_view = ImageGridView::new(
            model.clone(),
            Some(context_menu.clone()),
            Some(libclient_host.shared_ui_provider()),
        );
        let metadatapanecontroller = MetadataPaneController::new();
        let mut module = GridViewModule {
            imp_: ControllerImplCell::default(),
            selection_controller: selection_controller.clone(),
            context_menu,
            image_grid_view,
            metadatapanecontroller,
            widget,
        };

        module.build_widget();

        let module = Rc::new(module);
        <Self as Controller>::start(&module);

        module
    }

    fn build_widget(&mut self) {
        self.image_grid_view.set_vexpand(true);
        self.context_menu.set_parent(&*self.image_grid_view);
        self.context_menu.set_has_arrow(false);
        self.image_grid_view.connect_unrealize(glib::clone!(
            #[strong(rename_to = menu)]
            self.context_menu,
            move |_| {
                menu.unparent();
            }
        ));

        let gesture = gtk4::GestureClick::new();
        self.image_grid_view.add_controller(gesture.clone());
        let sender = self.sender();
        gesture.connect_pressed(glib::clone!(
            #[strong]
            sender,
            move |gesture, _, x, y| {
                let gesture = gesture.clone();
                send_async_local!(GridMsg::Click(gesture, x, y), sender);
            }
        ));
        let sender = self.sender();
        self.image_grid_view
            .add_rating_listener(Box::new(move |(id, rating)| {
                send_async_local!(GridMsg::ChangeRating(id, rating), sender);
            }));

        let scrollview = gtk4::ScrolledWindow::new();
        scrollview.set_child(Some(&*self.image_grid_view));
        scrollview.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
        self.widget.set_wide_handle(true);
        let box_ = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        box_.append(&scrollview);
        let toolbar = crate::niepce::ui::imagetoolbar::image_toolbar_new();
        box_.append(&toolbar);
        self.widget.set_start_child(Some(&box_));

        let dock = Dock::new();
        let sender = self.sender();
        self.metadatapanecontroller
            .set_forwarder(Some(Box::new(move |msg| match msg {
                MetadataOutputMsg::MetadataChanged(new, old) => {
                    send_async_local!(GridMsg::MetadataChanged(new, old), sender)
                }
            })));

        self.widget.set_end_child(Some(&dock));
        self.widget.set_resize_end_child(false);
        dock.vbox().append(self.metadatapanecontroller.widget());
    }

    fn on_librarylistview_click(&self, gesture: &gtk4::GestureClick, x: f64, y: f64) {
        let button = gesture.current_button();
        dbg_out!("GridView click handler, button: {button}");
        if button == 3
            && !self
                .image_grid_view
                .model()
                .map(|model| model.selection().is_empty())
                .unwrap_or(true)
        {
            self.context_menu
                .set_pointing_to(Some(&gdk4::Rectangle::new(x as i32, y as i32, 1, 1)));
            self.context_menu.popup();
        }
    }

    pub fn on_lib_notification(&self, ln: &LibNotification, client: &Arc<LibraryClient>) {
        match ln {
            LibNotification::MetadataQueried(lm) => {
                self.metadatapanecontroller.display(lm.id(), Some(lm));
            }
            LibNotification::MetadataChanged(lm) => {
                if lm.id != 0 && self.metadatapanecontroller.displayed() == lm.id {
                    client.request_metadata(lm.id);
                }
            }
            _ => (),
        }
    }

    pub fn display_none(&self) {
        self.metadatapanecontroller.display(0, None);
    }
}
