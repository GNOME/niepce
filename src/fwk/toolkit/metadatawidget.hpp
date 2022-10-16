/*
 * niepce - fwk/toolkit/metadatawidget.hpp
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

#include <map>
#include <string>
#include <optional>

#include <gtkmm/grid.h>
#include <gtkmm/textview.h>

#include "fwk/base/propertybag.hpp"
#include "fwk/toolkit/widgets/toolboxitemwidget.hpp"

namespace xmp {
struct MetaDataSectionFormat;
struct MetaDataFormat;
}

namespace fwk {

enum class MetaDT {
    NONE = 0,
    STRING,
    STRING_ARRAY,
    TEXT,
    DATE,
    FRAC,
    FRAC_DEC, // Fraction as decimal
    STAR_RATING,
    SIZE,     // Size in bytes
};

struct MetaDataFormat {
    const char * label;
    ffi::NiepcePropertyIdx id;
    MetaDT       type;
    bool         readonly;
};

struct MetaDataSectionFormat {
    const char * section;
    const std::vector<MetaDataFormat> formats;
};

class XmpMeta;
class TokenTextView;

class MetaDataWidget
	: public fwk::ToolboxItemWidget
{
public:
    MetaDataWidget(const Glib::ustring & title);

    void add_data(const MetaDataFormat& current,
                  std::optional<PropertyValuePtr>&& value);
    void set_data_format(const MetaDataSectionFormat* fmt);
    void set_data_source(const fwk::PropertyBagPtr& properties);

    sigc::signal<void(const fwk::PropertyBagPtr &, const fwk::PropertyBagPtr &)> signal_metadata_changed;
protected:
    bool on_str_changed(Gtk::Entry *, ffi::NiepcePropertyIdx prop);
    bool on_text_changed(Glib::RefPtr<Gtk::TextBuffer> b, ffi::NiepcePropertyIdx prop);
    bool on_string_array_changed(fwk::TokenTextView * ttv,
                                 ffi::NiepcePropertyIdx prop);
    void on_int_changed(int, ffi::NiepcePropertyIdx prop);
private:
    void clear_widget(const std::pair<const ffi::NiepcePropertyIdx, Gtk::Widget *> & p);
    void create_widgets_for_format(const MetaDataSectionFormat * fmt);

    // the widget factory
    Gtk::Widget* create_star_rating_widget(bool readonly, ffi::NiepcePropertyIdx id);
    Gtk::Widget* create_string_array_widget(bool readonly, ffi::NiepcePropertyIdx id);
    Gtk::Widget* create_text_widget(bool readonly, ffi::NiepcePropertyIdx id);
    Gtk::Widget* create_string_widget(bool readonly, ffi::NiepcePropertyIdx id);
    Gtk::Widget* create_date_widget(bool readonly, ffi::NiepcePropertyIdx id);

    // set data
    // Fraction as a decimal
    bool set_fraction_dec_data(Gtk::Widget* w, const PropertyValuePtr& value);
    // Fraction as fraction
    bool set_fraction_data(Gtk::Widget* w, const PropertyValuePtr& value);
    bool set_star_rating_data(Gtk::Widget* w, const PropertyValuePtr& value);
    bool set_string_array_data(Gtk::Widget* w, bool readonly, const PropertyValuePtr& value);
    bool set_text_data(Gtk::Widget* w, bool readonly,
                       const PropertyValuePtr& value);
    bool set_string_data(Gtk::Widget* w, bool readonly,
                         const PropertyValuePtr& value);
    bool set_date_data(Gtk::Widget* w, const PropertyValuePtr& value);

    void emit_metadata_changed(ffi::NiepcePropertyIdx prop, const fwk::PropertyValuePtr & value);

    Gtk::Grid    m_table;
    std::map<ffi::NiepcePropertyIdx, Gtk::Widget *> m_data_map;
    fwk::PropertyBagPtr m_current_data;
    const MetaDataSectionFormat * m_fmt;
    bool m_update;

    static void rating_callback(GtkWidget* w, gint rating, gpointer user_data);
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
