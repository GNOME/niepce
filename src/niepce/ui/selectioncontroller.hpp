/*
 * niepce - ui/selectioncontroller.h
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

#pragma once

#include <memory>
#include <optional>

#include <sigc++/signal.h>

#include "fwk/base/propertybag.hpp"
#include "fwk/toolkit/controller.hpp"
#include "ui/imageliststore.hpp"

#include "rust_bindings.hpp"

namespace Gtk {
class IconView;
class Widget;
}

namespace ui {

typedef ::rust::Box<SelectionController> SelectionControllerPtr;

/** interface for selectable image. Make the controller
 *  inherit/implement it.
 */
class IImageSelectable
{
public:
    typedef std::shared_ptr<IImageSelectable> Ptr;
    typedef std::weak_ptr<IImageSelectable> WeakPtr;

    virtual ~IImageSelectable() {}
    virtual Gtk::IconView * image_list() = 0;
    /** Return the id of the selection. <= 0 is none. */
    virtual eng::library_id_t get_selected() = 0;
    /** select the image a specific id
     *  might emit the signals.
     */
    virtual void select_image(eng::library_id_t id) = 0;
};

class SelectionController_2
    : public fwk::Controller
{
public:
    typedef std::shared_ptr<SelectionController_2> Ptr;

    SelectionController_2(const npc::LibraryClientHost& client)
        : m_ctrl(SelectionController_new(client))
        {}

    const SelectionControllerPtr& obj() const
        { return m_ctrl; }
    void add_selectable(const IImageSelectable::WeakPtr &);
private:
    SelectionControllerPtr m_ctrl;
    std::vector<IImageSelectable::WeakPtr> m_selectables;
};

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
