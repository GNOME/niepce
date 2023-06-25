/*
 * niepce - ncr/image.cpp
 *
 * Copyright (C) 2008-2023 Hubert Figuiere
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


extern "C" {
#include <babl/babl.h>
}

#include <gegl.h>

#include <gdkmm/memorytexture.h>

#include <libopenraw/libopenraw.h>

#include "fwk/base/debug.hpp"
#include "fwk/utils/pathutils.hpp"
#include "ncr.hpp"
#include "image.hpp"

namespace ncr {

struct Image::Private {
    Private(Image& self)
        : m_self(self)
        , m_status(Status::UNSET)
        , m_width(0)
        , m_height(0)
        , m_orientation(0), m_vertical(false)
        , m_flip(false)
        , m_tilt(0.0)
        , m_graph(nullptr), m_rgb(nullptr), m_rotate_n(nullptr)
        , m_scale(nullptr)
        , m_sink(nullptr), m_sink_buffer(nullptr)
        {
        }
    ~Private()
        {
            if(m_graph) {
                g_object_unref(m_graph);
            }
            if(m_sink_buffer) {
                g_object_unref(m_sink_buffer);
            }
        }

    /** Call this to initialise the reload process */
    void prepare_reload();

    /** continue the reload
     * @param node the node for the loaded image
     * @param orientation the exif orientation.
     */
    void reload_node(GeglNode* node, int orientation);

    Image& m_self;
    Status m_status;
    int m_width, m_height; /**< the native dimension, with orientation */
    int m_orientation;     /**< EXIF orientation in degrees */
    bool m_vertical;
    bool m_flip;           /**< EXIF flip */
    double m_tilt;         /**< User rotation */
    GeglNode* m_graph;
    GeglNode* m_rgb;    /**< RGB pixmap */
    GeglNode* m_rotate_n;
    GeglNode* m_scale;
    // sink
    GeglNode*   m_sink;
    GeglBuffer* m_sink_buffer;

    GeglNode* _rotate_node(int orientation);
    GeglNode* _scale_node();
    GeglNode* _load_dcraw(const std::string &path);
    GeglNode* _load_raw(const std::string &path);

    Glib::RefPtr<Gdk::Pixbuf> m_pixbuf_cache;
};

Image::Image()
    : priv(new Private(*this))
{
}

Image::~Image()
{
    delete priv;
}

GeglNode* Image::Private::_rotate_node(int orientation)
{
//    GeglNode* rotateNode = gegl_node_new_child(m_graph, nullptr, nullptr);

    DBG_OUT("rotation is %d", orientation);
    switch(orientation) {
    case 0:
    case 1:
        m_orientation = 0;
        m_vertical = false;
        m_flip = false;
        break;
    case 2:
        m_orientation = 0;
        m_vertical = false;
        m_flip = true;
        break;
    case 4:
        m_orientation = 180;
        m_vertical = false;
        m_flip = true;
        break;
    case 3:
        m_orientation = 180;
        m_vertical = false;
        m_flip = false;
        break;
    case 5:
        m_orientation = 270;
        m_vertical = true;
        m_flip = true;
        break;
    case 6:
        m_orientation = 270;
        m_vertical = true;
        m_flip = false;
        break;
    case 7:
        m_orientation = 90;
        m_vertical = true;
        m_flip = true;
        break;
    case 8:
        m_orientation = 90;
        m_vertical = true;
        m_flip = false;
        break;
    }

    // @todo ideally we would have a plain GEGL op for that.
    // @todo find a test case.
//    GeglNode* flipNode = gegl_node_new_child(rotateNode,
//                                             "operation", "gegl:reflect",
//                                             "x", (m_flip ? -1.0 : 1.0), nullptr);

    GeglNode* rotationNode = nullptr;
    double rotate = m_orientation + m_tilt;
    rotationNode = gegl_node_new_child(m_graph,
                                       "operation", "gegl:rotate",
                                       "degrees", rotate, nullptr);
//    gegl_node_link(flipNode, rotationNode);

    return rotationNode;
}

/** Use dcraw as a loader. Temporary fix */
GeglNode* Image::Private::_load_dcraw(const std::string &p)
{
    return gegl_node_new_child(m_graph,
                               "operation", "gegl:raw-load",
                               "path", p.c_str(), nullptr);
}

/** create the RAW pipeline */
GeglNode* Image::Private::_load_raw(const std::string &p)
{
    GeglNode* rgb = nullptr;

    ORRawDataRef rawdata;
    or_get_extract_rawdata(p.c_str(), 0, &rawdata);
    GeglBuffer* buffer = ncr::load_rawdata(rawdata);
    // @todo can return a nullptr buffer if load failed. Deal with that.
    GeglNode* load_file = gegl_node_new_child(m_graph,
                                              "operation", "gegl:buffer-source",
                                              "buffer", buffer, nullptr);
    or_cfa_pattern pattern_type = or_rawdata_get_cfa_pattern_type(rawdata);
    or_rawdata_release(rawdata);

    GeglNode* stretch =
        gegl_node_new_child(m_graph,
                            "operation", "gegl:stretch-contrast",
                            nullptr);
    GeglNode* demosaic =
        gegl_node_new_child(m_graph,
                            "operation", "gegl:demosaic-bimedian",
                            nullptr);
    // @todo refactor somewhere.
    int npattern = 0;
    switch(pattern_type) {
    case OR_CFA_PATTERN_GRBG:
        npattern = 0;
        break;
    case OR_CFA_PATTERN_BGGR:
        npattern = 1;
        break;
    case OR_CFA_PATTERN_GBRG:
        npattern = 2;
        break;
    case OR_CFA_PATTERN_RGGB:
        npattern = 3;
        break;
    default:
        break;
    }
    gegl_node_set(demosaic, "pattern", npattern, nullptr);

    gegl_node_link_many(load_file, stretch, demosaic, nullptr);

    rgb = demosaic;
    return rgb;
}

GeglNode* Image::Private::_scale_node()
{
    auto scale = 1.0;
    return gegl_node_new_child(m_graph,
                               "operation", "gegl:scale-ratio",
                               "x", scale, "y", scale, nullptr);
}


void Image::Private::prepare_reload()
{
    m_status = Status::LOADING;
    m_pixbuf_cache.reset();

    if(m_graph) {
        g_object_unref(m_graph);
    }
    m_graph = gegl_node_new();
//    m_graph->set("format", babl_format("RGB u16"));
}

void Image::reload_pixbuf(const Glib::RefPtr<Gdk::Pixbuf> & p)
{
    priv->prepare_reload();
    priv->m_pixbuf_cache = p;
    GeglNode* load_file = gegl_node_new_child(priv->m_graph,
                                              "operation", "gegl:pixbuf",
                                              "pixbuf", p->gobj(), nullptr);

    priv->reload_node(load_file, 0);
}

void Image::reload(const std::string & p, bool is_raw,
    int orientation)
{
    priv->prepare_reload();

    GeglNode* load_file = nullptr;

    DBG_OUT("loading file %s", p.c_str());

    if (!fwk::path_exists(p)) {
        set_status(Status::NOT_FOUND);
    } else if (!is_raw) {
        load_file = gegl_node_new_child(priv->m_graph,
                                        "operation", "gegl:load",
                                        "path", p.c_str(), nullptr);
    } else {
        load_file = priv->_load_dcraw(p);
    }
    priv->reload_node(load_file, orientation);
}

void Image::Private::reload_node(GeglNode* node, int orientation)
{
    int width, height;
    width = height = 0;

    if (node) {
        DBG_ASSERT(m_status == Status::LOADING,
                   "prepare_reload() might not have been called");

        m_rotate_n = _rotate_node(orientation);
        m_scale = _scale_node();
        m_sink = gegl_node_create_child (m_graph, "gegl:display");

//      gegl_node_new_child(m_graph,
//                          "operation", "gegl:buffer-sink",
//                          "format", babl_format("RGB u8"),
//                          "buffer", &(m_sink_buffer), nullptr);

        gegl_node_link_many(node, m_rotate_n,
                            m_scale, m_sink, nullptr);

//      gegl_node_process(m_sink);
        // DEBUG
#if 0
        GeglNode* debugsink;
        GeglNode* graph =
            gegl_graph (debugsink=gegl_node ("gegl:save",
                                             "path", "/tmp/gegl.png", nullptr,
                                         gegl_node ("gegl:buffer-source",
                                                    "buffer",
                                                    m_sink_buffer,
                                                    nullptr)
                            )
                );
        gegl_node_process (debugsink);
        g_object_unref (graph);
#endif
        // END DEBUG

        GeglRectangle rect = gegl_node_get_bounding_box(node);
        width = rect.width;
        height = rect.height;
    }
    DBG_OUT("width %d height %d - status %d", width, height, (int)m_status);
    if (m_status < Status::ERROR && (!width || !height)) {
        m_status = Status::ERROR;
    }
    if(m_vertical) {
        m_width = height;
        m_height = width;
    }
    else {
        m_width = width;
        m_height = height;
    }

    m_self.signal_update();
}

void Image::set_output_scale(double scale)
{
    DBG_OUT("scale %f", scale);
    if(!priv->m_scale) {
        DBG_OUT("nothing loaded");
        return;
    }
    gegl_node_set(priv->m_scale, "x", scale, "y", scale, nullptr);

    signal_update();
}



void Image::set_tilt(double angle)
{
    if(!priv->m_rotate_n) {
        DBG_OUT("nothing loaded");
        return;
    }
    priv->m_tilt = angle;
    gegl_node_set(priv->m_rotate_n, "degrees",
                  priv->m_orientation + priv->m_tilt, nullptr);

    signal_update();
}


void Image::rotate_left()
{
    rotate_by(-90);
}

void Image::rotate_right()
{
    rotate_by(90);
}

void Image::rotate_half()
{
    rotate_by(180);
}


void Image::rotate_by(int degree)
{
    priv->m_orientation += degree;
    if(priv->m_orientation < 0) {
        // negative values aren't good
        priv->m_orientation += 360;
    }
    else {
        // within 0..359 degrees anyway
        priv->m_orientation %= 360;
    }
    gegl_node_set(priv->m_rotate_n, "degrees",
                  priv->m_orientation + priv->m_tilt, nullptr);
    signal_update();
}

bool Image::to_buffer(rust::Slice<uint8_t> buffer)
{
    if (get_status() == Status::ERROR) {
        // return the error
        DBG_OUT("error");
        return false;
    }
    if (!priv->m_sink) {
        DBG_OUT("nothing loaded");
        return false;
    }
    DBG_ASSERT(get_status() == Status::LOADING, "wrong status for image");
    DBG_OUT("processing");
    gegl_node_process(priv->m_scale);
    GeglRectangle roi = gegl_node_get_bounding_box(priv->m_scale);
    auto w = roi.width;
    auto h = roi.height;

    DBG_OUT("texture w=%d, h=%d", w, h);

    const Babl* format = babl_format("R'G'B' u8");
    DBG_ASSERT((std::size_t)(w * h * 3) == buffer.size(),
               "wrong buffer size");
    gegl_node_blit(priv->m_scale, 1.0, &roi, format, buffer.data(), 0,
                   (GeglBlitFlags)(GEGL_BLIT_CACHE | GEGL_BLIT_DIRTY));

    set_status(Status::LOADED);
    return true;
}

Status Image::get_status() const
{
    return priv->m_status;
}

void Image::set_status(Status status)
{
    priv->m_status = status;
}

int Image::get_original_width() const
{
    return priv->m_width;
}


int Image::get_original_height() const
{
    return priv->m_height;
}


int Image::get_output_width() const
{
    if (!priv->m_scale) {
        ERR_OUT("scale is null");
        return 0;
    }
    GeglRectangle roi = gegl_node_get_bounding_box(priv->m_scale);
    return roi.width;
}


int Image::get_output_height() const
{
    if (!priv->m_scale) {
        ERR_OUT("scale is null");
        return 0;
    }
    GeglRectangle roi = gegl_node_get_bounding_box(priv->m_scale);
    return roi.height;
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
