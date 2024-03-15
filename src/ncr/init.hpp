/*
 * niepce - ncr/init.hpp
 *
 * Copyright (C) 2011 Hub Figuiere
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

#include <gegl.h>
#include <babl/babl.h>

#include <cstdint>
#include <string>

namespace ncr {

void init();

inline
GeglNode* gegl_node_new_child(GeglNode* node,
                              const char* prop1, const uint8_t* value1,
                              const uint8_t* prop2, const uint8_t* value2)
{
    return ::gegl_node_new_child(node, prop1, value1, prop2, value2, nullptr);
}

inline
GeglNode* gegl_node_new_child_so(GeglNode* node,
                                 const char* prop1, const uint8_t* value1,
                                 const uint8_t* prop2, GObject *value2)
{
    return ::gegl_node_new_child(node, prop1, value1, prop2, value2, nullptr);
}

inline
GeglNode* gegl_node_new_child_sf(GeglNode* node,
                                 const char* prop1, const uint8_t* value1,
                                 const uint8_t* prop2, double value2)
{
    return ::gegl_node_new_child(node, prop1, value1, prop2, value2, nullptr);
}

inline
GeglNode* gegl_node_new_child_sff(GeglNode* node,
                                 const char* prop1, const uint8_t* value1,
                                 const uint8_t* prop2, double value2,
                                 const uint8_t* prop3, double value3)
{
    return ::gegl_node_new_child(node, prop1, value1, prop2, value2, prop3, value3, nullptr);
}

inline
GeglNode* gegl_node_create_child(GeglNode *node, const char* op) {
    return ::gegl_node_create_child(node, op);
}

using ::gegl_node_new;
using ::gegl_node_process;

inline void
gegl_node_link_many(GeglNode* node, GeglNode* node1, GeglNode* node2, GeglNode* node3)
{
    ::gegl_node_link_many(node, node1, node2, node3, nullptr);
}

inline int32_t
gegl_node_get_bounding_box_w(GeglNode* node)
{
    return ::gegl_node_get_bounding_box(node).width;
}

inline int32_t
gegl_node_get_bounding_box_h(GeglNode* node)
{
    return ::gegl_node_get_bounding_box(node).height;
}

inline void
gegl_node_set(GeglNode* node, const char* prop1, double val1,
              const uint8_t* prop2, double val2)
{
    ::gegl_node_set(node, prop1, val1, prop2, val2, nullptr);
}

struct GeglRectangle_;

inline void
gegl_node_blit (GeglNode* node, gdouble scale,
                const GeglRectangle_& roi,
                const Babl* format, uint8_t* destination_buf,
                gint rowstride, int32_t flags)
{
    ::gegl_node_blit(node, scale, (const ::GeglRectangle*)&roi,
                     format, (gpointer)destination_buf, rowstride,
                     (GeglBlitFlags)flags);
}


using ::babl_format;
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
