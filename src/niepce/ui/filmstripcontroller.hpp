/*
 * niepce - niepce/ui/filmstripcontroller.hpp
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

#include "fwk/toolkit/uicontroller.hpp"
#include "fwk/toolkit/notificationcenter.hpp"
#include "ui/selectioncontroller.hpp"

namespace Gtk {
	class IconView;
}

namespace ui {

class FilmStripController
	: public fwk::UiController,
	  public IImageSelectable
{
public:
	typedef std::shared_ptr<FilmStripController> Ptr;
	typedef std::weak_ptr<FilmStripController> WeakPtr;

	FilmStripController(const ImageListStorePtr& store);

	virtual Gtk::IconView * image_list() override;
	virtual eng::library_id_t get_selected() override;
	virtual void select_image(eng::library_id_t id) override;

	virtual Gtk::Widget * buildWidget() override;

private:
	std::shared_ptr<ffi::ThumbStripView> m_thumb_strip_view;
	Gtk::IconView * m_thumbview;
	ImageListStorePtr m_store;
};


}
/*
  Local Variables:
  mode:c++
  c-basic-offset:4
  tab-width:4
  c-file-style:"stroustrup"
  c-file-offsets:((innamespace . 0))
  indent-tabs-mode:t
  fill-column:99
  End:
*/
