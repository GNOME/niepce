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

#include <gtk/gtk.h>

#include <sigc++/signal.h>

#include "fwk/base/propertybag.hpp"
#include "fwk/toolkit/controller.hpp"
#include "ui/imageliststore.hpp"

namespace Gtk {
class IconView;
class Widget;
}

namespace ui {

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


class SelectionController
    : public fwk::Controller
{
public:
    typedef std::shared_ptr<SelectionController> Ptr;
    SelectionController();

    void add_selectable(const IImageSelectable::WeakPtr &);

    void activated(const Gtk::TreeModel::Path & /*path*/,
                   const IImageSelectable::WeakPtr & selectable);
    void selected(const IImageSelectable::WeakPtr &);


    const ImageListStorePtr& get_list_store() const
        { return m_imageliststore; }

    // the signal to call when selection is changed.
    sigc::signal<void(eng::library_id_t)> signal_selected;

    // signal for when the item is activated (ie double-click)
    sigc::signal<void(eng::library_id_t)> signal_activated;

    /////////
    /** select the previous image. Emit the signal */
    void select_previous();
    /** select the next image. Emit the signal */
    void select_next();
    /** rotate the image in selection by %angle (trigonometric) */
    void rotate(int angle);
    /** set the rating of selection to %rating. */
    void set_rating(int rating);
    /** set the label of selection to the label with index %label. */
    void set_label(int label);
    /** set flag */
    void set_flag(int flag);

    void set_property(ffi::NiepcePropertyIdx idx, int value);

    void set_properties(const fwk::PropertyBagPtr & props,
                        const fwk::PropertyBagPtr & old);

    /** the content will change */
    void content_will_change();
    /** Write the file(s) metadata to disk. */
    void write_metadata();
    void move_to_trash();

    /** get the current selection
     *  todo: change it to support multiple
     */
    eng::library_id_t get_selection() const;
    eng::LibFilePtr get_file(eng::library_id_t id) const;
protected:
    virtual void _added() override;
private:
    libraryclient::LibraryClientPtr getLibraryClient();

    bool _set_metadata(const std::string & undo_label,
                       eng::library_id_t file_id,
                       ffi::NiepcePropertyIdx meta,
                       int old_value, int new_value);
    bool _set_metadata(const std::string & undo_label,
                       eng::library_id_t file_id,
                       const fwk::PropertyBagPtr & props,
                       const fwk::PropertyBagPtr & old);
    /** move the selection and emit the signal
     * @param backwards true if the move is backwards.
     */
    void _selection_move(bool backwards);

    ImageListStorePtr  m_imageliststore;
    bool m_in_handler;
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
