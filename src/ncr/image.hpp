/*
 * niepce - ncr/image.h
 *
 * Copyright (C) 2008-2022 Hubert Figuière
 *
 * This program is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 3 of the License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this program; if not, see
 * <http://www.gnu.org/licenses/>.
 */


#pragma once

#include <memory>

#include <gdkmm/pixbuf.h>

namespace ncr {

class Image
//    : public std::enable_shared_from_this<Image>
{
public:
    typedef std::shared_ptr<Image> Ptr;
    enum class Status {
        UNSET = 0,
        LOADING,
        LOADED,
        ERROR,
        NOT_FOUND,
        _LAST
    };

    Image();
    virtual ~Image();

    /* get a cairo surface to display the resulting image */
    Cairo::RefPtr<Cairo::ImageSurface> cairo_surface_for_display();

    /** The status of the image. */
    Status get_status() const;
    void set_status(Status s);

    /* the dimensions of the original image */
    int get_original_width() const;
    int get_original_height() const;

    /* the dimension of the output image, after scale */
    int get_output_width() const;
    int get_output_height() const;

    void reload(const std::string & p, bool is_raw,
        int orientation);
    void reload(const Glib::RefPtr<Gdk::Pixbuf> & p);
    /** set the output scale */
    void set_output_scale(double scale);

    /** tile the image in degrees. */
    void set_tilt(double angle);

    /** rotate the image left */
    void rotate_left();
    /** rotate the image right */
    void rotate_right();
    /** rotate 180 degres */
    void rotate_half();

    void set_color_temp(int temp);
    void set_exposure(double exposure);
    void set_brightness(int brightness);
    void set_contrast(int contrast);
    void set_saturation(int saturation);
    void set_vibrance(int vibrance);

    /** this signal is emitted each time the
        image is changed. */
    sigc::signal<void(void)> signal_update;
private:

    /** rotate by x degrees (orientation)
     *  ensure the end results is within 0..359.
     */
    void rotate_by(int degree);

    class Private;
    Private *priv;
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
