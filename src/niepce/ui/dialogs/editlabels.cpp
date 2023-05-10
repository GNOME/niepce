/*
 * niepce - niepce/ui/dialogs/editlabels.cpp
 *
 * Copyright (C) 2009-2023 Hubert Figuière
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

#include <algorithm>

#include <boost/format.hpp>

#include <glibmm/i18n.h>
#include <gtkmm/entry.h>
#include <gtkmm/label.h>

#include "fwk/base/debug.hpp"
#include "fwk/toolkit/application.hpp"
#include "fwk/toolkit/gdkutils.hpp"
#include "editlabels.hpp"

using eng::LibraryClientPtr;

namespace ui {

EditLabels::EditLabels(const eng::LibraryClientHost& libclient)
    : fwk::Dialog("/net/figuiere/Niepce/ui/editlabels.ui", "editLabels")
    , m_lib_client(libclient)
{
    auto& provider = libclient.getDataProvider();
    auto count = provider.label_count();
    for (size_t i = 0; i < count; i++) {
        m_labels.push_back(rust::Box<eng::Label>::from_raw(provider.label_at(i)));
    }

    std::fill(m_status.begin(), m_status.end(), false);
}

void EditLabels::run_modal(GtkWindow* parent, rust::Fn<void(EditLabelsPtr, int32_t)> on_ok, EditLabelsPtr self) const
{
    run_modal_(parent, [self, on_ok] (int32_t r) { on_ok(self, r); });
}

void EditLabels::setup_widget()
{
    Glib::RefPtr<Gtk::Builder> _builder = builder();
    DBG_OUT("setup Edit Labels dialog");
    add_header(_("Edit Labels"));

    const char * colour_fmt = "colorbutton%1%";
    const char * value_fmt = "value%1%";
    for (size_t i = 0; i < NUM_LABELS; i++) {
        bool has_label = m_labels.size() > i;

        Gtk::ColorButton *colourbutton;
        Gtk::Entry *labelentry;

        m_colours[i] = colourbutton = _builder->get_widget<Gtk::ColorButton>(str(boost::format(colour_fmt) % (i+1)));
        DBG_ASSERT(labelentry, "couldn't find label");
        m_entries[i] = labelentry = _builder->get_widget<Gtk::Entry>(str(boost::format(value_fmt) % (i+1)));

        if(has_label) {
            Gdk::RGBA colour = fwk::rgbcolour_to_gdkcolor(m_labels[i]->colour());
            colourbutton->set_rgba(colour);
            labelentry->set_text(std::string(m_labels[i]->label()));
        }
        colourbutton->signal_color_set().connect(
            sigc::bind(sigc::mem_fun(*this, &EditLabels::label_colour_changed), i));
        labelentry->signal_changed().connect(
            sigc::bind(sigc::mem_fun(*this, &EditLabels::label_name_changed), i));
    }
    DBG_OUT("all colours setup");
    gtkDialog().signal_response().connect(sigc::mem_fun(*this, &EditLabels::update_labels));
}


void EditLabels::label_name_changed(size_t idx)
{
    m_status[idx] = true;
}


void EditLabels::label_colour_changed(size_t idx)
{
    m_status[idx] = true;
}

void EditLabels::update_labels(int /*response*/)
{
    rust::Box<fwk::UndoTransaction> undo = fwk::UndoTransaction_new(_("Change Labels"));
    for(size_t i = 0; i < 5; i++) {
        if(m_status[i]) {
            bool has_label = m_labels.size() > i;

            DBG_OUT("updating label %lu", i);
            std::string new_name = m_entries[i]->get_text();
            // a newname is NOT valid.
            if(new_name.empty()) {
                continue;
            }
            std::string new_colour(fwk::gdkcolor_to_rgbcolour(m_colours[i]->get_rgba())->to_string());

            auto client = &m_lib_client.client();
            if(has_label) {
                std::string current_name(m_labels[i]->label());
                std::string current_colour(m_labels[i]->colour().to_string());
                auto label_id = m_labels[i]->id();

                auto command = fwk::UndoCommand_new(
                    std::make_unique<fwk::RedoFnVoid>(
                        [client, new_name, new_colour, label_id] () {
                            client->update_label(label_id, new_name, new_colour);
                        }),
                    std::make_unique<fwk::UndoFnVoid>(
                        [client, current_name, current_colour, label_id] () {
                            client->update_label(label_id, current_name, current_colour);
                        })
                    );
                undo->add(std::move(command));
            } else {
                auto command = fwk::UndoCommand_new_int(
                    std::make_unique<fwk::RedoFnInt>(
                        [client, new_name, new_colour] () {
                            return client->create_label_sync(new_name, new_colour);
                        }),
                    std::make_unique<fwk::UndoFnInt>(
                        [client] (int64_t label) {
                            client->delete_label(label);
                        })
                    );
                undo->add(std::move(command));
            }
        }
    }
    if (!undo->is_empty()) {
        undo->execute();
        fwk::Application::app()->begin_undo(std::move(undo));
    }
}

/*
  Local Variables:
  mode:c++
  c-file-style:"stroustrup"
  c-file-offsets:((innamespace . 0))
  indent-tabs-mode:nil
  fill-column:80
  End:
*/
}
