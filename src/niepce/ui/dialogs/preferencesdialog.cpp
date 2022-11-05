/*
 * niepce - ui/dialogs/preferencesdialog.cpp
 *
 * Copyright (C) 2009-2022 Hubert Figuière
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

#include <glibmm/i18n.h>
#include <gtkmm/checkbutton.h>

#include "fwk/toolkit/configdatabinder.hpp"
#include "fwk/toolkit/application.hpp"
#include "fwk/toolkit/gtkutils.hpp"


#include "preferencesdialog.hpp"

namespace ui {

void PreferencesDialog::setup_widget()
{
    if(m_is_setup) {
        return;
    }

    add_header(_("Preferences"));

    Gtk::CheckButton* theme_checkbutton = nullptr;
    Gtk::CheckButton* reopen_checkbutton = nullptr;
    Gtk::CheckButton* write_xmp_checkbutton = nullptr;
    m_binder_pool = std::make_unique<fwk::DataBinderPool>();

    theme_checkbutton = builder()->get_widget<Gtk::CheckButton>("dark_theme_checkbox");
    theme_checkbutton->set_active(fwk::Application::app()
                            ->get_use_dark_theme());
    auto app = fwk::Application::app();
    theme_checkbutton->signal_toggled().connect(
        [app, theme_checkbutton] () {
            app->set_use_dark_theme(theme_checkbutton->property_active());
        });

    reopen_checkbutton = builder()->get_widget<Gtk::CheckButton>("reopen_checkbutton");
    m_binder_pool->add_binder(new fwk::ConfigDataBinder<bool>(
				    reopen_checkbutton->property_active(),
				    fwk::Application::app()->config(),
				    "reopen_last_catalog"));
    write_xmp_checkbutton = builder()->get_widget<Gtk::CheckButton>("write_xmp_checkbutton");
    m_binder_pool->add_binder(new fwk::ConfigDataBinder<bool>(
				  write_xmp_checkbutton->property_active(),
				  fwk::Application::app()->config(),
				  "write_xmp_automatically"));
    m_is_setup = true;
}


}

