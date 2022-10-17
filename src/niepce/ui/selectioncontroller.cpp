/*
 * niepce - niepce/ui/selectioncontroller.cpp
 *
 * Copyright (C) 2008-2022 Hubert Figuière
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

#include <gtkmm/iconview.h>
#include <gtkmm/treeiter.h>
#include <glibmm/i18n.h>

#include "fwk/base/autoflag.hpp"
#include "fwk/base/debug.hpp"
#include "fwk/toolkit/undo.hpp"
#include "fwk/toolkit/command.hpp"
#include "fwk/toolkit/application.hpp"
#include "engine/db/metadata.hpp"
#include "engine/db/properties.hpp"
#include "libraryclient/libraryclient.hpp"
#include "niepcewindow.hpp"
#include "selectioncontroller.hpp"

namespace ui {


SelectionController::SelectionController()
    : m_in_handler(false)
{
    m_imageliststore = ImageListStore::create();
}

void SelectionController::_added()
{
    m_imageliststore->set_parent_controller(m_parent);
}

void SelectionController::add_selectable(const IImageSelectable::WeakPtr & selectableWeak)
{
    auto selectable = selectableWeak.lock();
    DBG_OUT("added %p", (void*)selectable.get());
    m_selectables.push_back(selectableWeak);
    selectable->image_list()->signal_selection_changed().connect(
        sigc::bind(
            sigc::mem_fun(*this, &SelectionController::selected),
                   selectableWeak));
    selectable->image_list()->signal_item_activated().connect(
        sigc::bind(
            sigc::mem_fun(*this, &SelectionController::activated),
                   selectableWeak));
}


void SelectionController::activated(const Gtk::TreeModel::Path & path,
                                    const IImageSelectable::WeakPtr & /*selectable*/)
{
    fwk::AutoFlag f(m_in_handler);
    auto selection = m_imageliststore->get_libfile_id_at_path(path);
    if (selection) {
        DBG_OUT("item activated %Ld", (long long)selection);
        signal_activated(selection);
    }
}

void SelectionController::selected(const IImageSelectable::WeakPtr & selectableWeak)
{
    if(m_in_handler) {
        DBG_OUT("%p already in handler", (void*)this);
        return;
    }

    fwk::AutoFlag f(m_in_handler);

    auto selectable = selectableWeak.lock();
    if (!selectable) {
        // it can be, but at that point we should be terminating the app.
        ERR_OUT("selectable is null");
        return;
    }

    auto selection = selectable->get_selected();
    for(auto iterWeak : m_selectables) {
        auto iter = iterWeak.lock();
        if(iter && iter != selectable) {
            iter->select_image(selection);
        }
    }
    signal_selected(selection);
}


libraryclient::LibraryClientPtr SelectionController::getLibraryClient()
{
    ModuleShell::Ptr shell = std::dynamic_pointer_cast<ModuleShell>(m_parent.lock());
    DBG_ASSERT(static_cast<bool>(shell), "parent not a ModuleShell");
    return shell->getLibraryClient();
}

eng::library_id_t SelectionController::get_selection() const
{
    DBG_ASSERT(!m_selectables.empty(), "selectables list can't be empty");

    auto selectable = m_selectables[0].lock();
    DBG_ASSERT(static_cast<bool>(selectable), "selectable is null");

    return selectable->get_selected();
}

std::optional<eng::LibFilePtr> SelectionController::get_file(eng::library_id_t id) const
{
    return m_imageliststore->get_file(id);
}

void SelectionController::_selection_move(bool backwards)
{
    auto selection = get_selection();
    auto iter = m_imageliststore->get_iter_from_id(selection);
    if(!iter) {
        return;
    }

    if(backwards) {
        if(iter != m_imageliststore->gobjmm()->children().begin()) {
            --iter;
        }
    }
    else {
        iter++;
    }
    if (iter) {
        // make sure the iterator is valid...
        auto path = Gtk::TreePath(iter);
        auto file_id = m_imageliststore->get_libfile_id_at_path(path);

        fwk::AutoFlag f(m_in_handler);

        std::for_each(m_selectables.begin(), m_selectables.end(),
                      [file_id] (const IImageSelectable::WeakPtr & s) {
                          if (!s.expired()) {
                              s.lock()->select_image(file_id);
                          }
                      });
        signal_selected(file_id);
    }
}

/** select the previous image. Emit the signal */
void SelectionController::select_previous()
{
    _selection_move(true);
}


/** select the next image. Emit the signal */
void SelectionController::select_next()
{
    _selection_move(false);
}


void SelectionController::rotate(int angle)
{
    DBG_OUT("angle = %d", angle);
    auto selection = get_selection();
    if(selection >= 0) {
        Gtk::TreeModel::iterator iter = m_imageliststore->get_iter_from_id(selection);
        if(iter) {
            // @todo
        }
    }
}


bool SelectionController::_set_metadata(const std::string & undo_label,
                                        eng::library_id_t file_id,
                                        ffi::NiepcePropertyIdx meta,
                                        int old_value, int new_value)
{
    std::shared_ptr<fwk::UndoTransaction> undo =
        fwk::Application::app()->begin_undo(undo_label);

    auto libclient = getLibraryClient();
    undo->new_command<void>(
        [libclient, file_id, meta, new_value] () {
            ffi::libraryclient_set_metadata(
                libclient->client(), file_id, meta, &*fwk::property_value_new_int(new_value));
        },
        [libclient, file_id, meta, old_value] () {
            ffi::libraryclient_set_metadata(
                libclient->client(), file_id, meta, &*fwk::property_value_new_int(old_value));
        });
    undo->execute();
    return true;
}

bool SelectionController::_set_metadata(const std::string & undo_label,
                                        eng::library_id_t file_id,
                                        const fwk::WrappedPropertyBagPtr& props,
                                        const fwk::WrappedPropertyBagPtr& old)
{
    auto undo = fwk::Application::app()->begin_undo(undo_label);
    auto len = ffi::fwk_property_bag_len(props.get());
    for (size_t i = 0; i < len; i++) {
        auto key = ffi::fwk_property_bag_key_by_index(props.get(), i);
        // This is a shared_ptr because it's the only way to pass it to the lambda
        // as Box<> isn't copyable
        auto value = std::make_shared<fwk::PropertyValuePtr>(fwk::wrapped_property_bag_value(old, key));
        /*
        if (!result.empty()) {
            value = result.unwrap();
            if (!fwk_property_value_is_empty(value.get())) {
                // XXX
                //DBG_ASSERT(value.get_variant().type() == iter.second.get_variant().type(),
                //           "Value type mismatch");
            }
        }
        */

        auto libclient = getLibraryClient();
        auto new_value = std::make_shared<fwk::PropertyValuePtr>(fwk::wrapped_property_bag_value(props, key));
        undo->new_command<void>(
            [libclient, file_id, key, new_value] () {
                ffi::libraryclient_set_metadata(
                    libclient->client(), file_id, static_cast<ffi::NiepcePropertyIdx>(key), &**new_value);
            },
            [libclient, file_id, key, value] () {
                ffi::libraryclient_set_metadata(
                    libclient->client(), file_id, static_cast<ffi::NiepcePropertyIdx>(key), &**value);
            });
    }
    undo->execute();
    return true;
}

void SelectionController::set_label(int label)
{
    set_property(ffi::NiepcePropertyIdx::NpXmpLabelProp, label);
}


void SelectionController::set_rating(int rating)
{
    set_property(ffi::NiepcePropertyIdx::NpXmpRatingProp, rating);
}

void SelectionController::set_flag(int flag)
{
    set_property(ffi::NiepcePropertyIdx::NpNiepceFlagProp, flag);
}

void SelectionController::set_property(ffi::NiepcePropertyIdx idx, int value)
{
    DBG_OUT("property %u = %d", static_cast<uint32_t>(idx), value);
    eng::library_id_t selection = get_selection();
    if(selection >= 0) {
        std::optional<eng::LibFilePtr> f = m_imageliststore->get_file(selection);
        if (!f) {
            ERR_OUT("requested file %ld not found!", selection);
            return;
        }
        auto& file = f.value();
        DBG_OUT("old property is %d", file->property((uint32_t)idx));
        int32_t old_value = file->property((uint32_t)idx);
        const char *action = nullptr;
        switch(idx) {
        case ffi::NiepcePropertyIdx::NpNiepceFlagProp:
            action = _("Set Flag");
            break;
        case ffi::NiepcePropertyIdx::NpXmpRatingProp:
            action = _("Set Rating");
            break;
        case ffi::NiepcePropertyIdx::NpXmpLabelProp:
            action = _("Set Label");
            break;
        default:
            action = _("Set Property");
            break;
        }
        _set_metadata(action, selection, idx, old_value, value);
        // we need to set the property here so that undo/redo works
        // consistently.
        file->set_property((uint32_t)idx, value);
    }
}

void SelectionController::set_properties(const fwk::WrappedPropertyBagPtr& props,
                                         const fwk::WrappedPropertyBagPtr& old)
{
    eng::library_id_t selection = get_selection();
    if (selection >= 0) {
        _set_metadata(_("Set Properties"), selection, props, old);
    }
}

void SelectionController::content_will_change()
{
    m_imageliststore->clear_content();
}

void SelectionController::write_metadata()
{
    eng::library_id_t selection = get_selection();
    if(selection >= 0) {
        ffi::libraryclient_write_metadata(getLibraryClient()->client(), selection);
    }
}

void SelectionController::move_to_trash()
{
    eng::library_id_t trash_folder =
        ffi::libraryclient_get_trash_id(getLibraryClient()->client());
    eng::library_id_t selection = get_selection();
    if(selection >= 0) {
        auto f = m_imageliststore->get_file(selection);
        if (f) {
            auto& file = f.value();
            eng::library_id_t from_folder = file->folder_id();
            std::shared_ptr<fwk::UndoTransaction> undo =
                fwk::Application::app()->begin_undo(_("Move to Trash"));

            auto libclient = getLibraryClient();
            undo->new_command<void>(
                [libclient, selection, from_folder, trash_folder] () {
                    ffi::libraryclient_move_file_to_folder(
                        libclient->client(), selection, from_folder, trash_folder);
                },
                [libclient, selection, from_folder, trash_folder] () {
                    ffi::libraryclient_move_file_to_folder(
                        libclient->client(), selection, trash_folder, from_folder);
                });
            undo->execute();
        }
    }
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
