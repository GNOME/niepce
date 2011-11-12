/*
 * niepce - fwk/toolkit/widgets/ratinglabel.cpp
 *
 * Copyright (C) 2011 Hubert Figuiere
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


#include <cmath>

#include "fwk/base/debug.hpp"
#include "ratinglabel.hpp"


namespace fwk {

#ifndef DATADIR
#error DATADIR is not defined
#endif



const Cairo::RefPtr<Cairo::ImageSurface> & RatingLabel::get_star()
{
  static Cairo::RefPtr<Cairo::ImageSurface>  s_star;
  if(!s_star) {
    s_star = Cairo::ImageSurface::create_from_png(
		   std::string(DATADIR"/niepce/pixmaps/niepce-set-star.png"));
  }
  return s_star;
}

const Cairo::RefPtr<Cairo::ImageSurface> & RatingLabel::get_unstar()
{
  static Cairo::RefPtr<Cairo::ImageSurface> s_unstar;
  if(!s_unstar) {
    s_unstar = Cairo::ImageSurface::create_from_png(
            std::string(DATADIR"/niepce/pixmaps/niepce-unset-star.png"));
  }
  return s_unstar;
}


void RatingLabel::draw_rating(const Cairo::RefPtr<Cairo::Context> & cr, 
			      int32_t rating,
			      const Cairo::RefPtr<Cairo::ImageSurface> & star,
			      const Cairo::RefPtr<Cairo::ImageSurface> & unstar,
			      double x, double y)
{
    if(!star || !unstar) {
        return;
    }
    if(rating == -1) {
        rating = 0;
    }
    int w = star->get_width();
    int h = star->get_height();
    y -= h;
    for(int32_t i = 1; i <= 5; i++) {
        if(i <= rating) {
            cr->set_source(star, x, y);
        }
        else {
            cr->set_source(unstar, x, y);
        }
        cr->paint();
        x += w;
    }
}


void RatingLabel::get_geometry(double & w, double & h)
{
  w = get_star()->get_width() * 5;
  h = get_star()->get_height();
}

int RatingLabel::rating_value_from_hit_x(double x)
{
  double width = get_star()->get_width();
  return round(x / width);
}

  RatingLabel::RatingLabel(int rating, bool editable)
  : Gtk::DrawingArea()
  , m_rating(rating)
  , m_is_editable(editable)
{
  DBG_OUT("ctor");
}

RatingLabel::~RatingLabel()
{
  DBG_OUT("dtor");
}

void RatingLabel::set_editable(bool editable)
{
  m_is_editable = editable;
  // TODO change the GdkEventMask if the widget is realized.
}

void RatingLabel::set_rating(int rating)
{
  if(rating != m_rating) {
    m_rating = rating;
    queue_draw();
  }
}


void RatingLabel::on_realize()
{
  Gtk::Widget::on_realize();
  if(m_is_editable) {
    Glib::RefPtr<Gdk::Window> win = get_window();
    Gdk::EventMask mask = win->get_events();
    mask |= Gdk::BUTTON_PRESS_MASK;
    //    | Gdk::BUTTON_RELEASE_MASK 
    //    | Gdk::BUTTON1_MOTION_MASK;
    win->set_events(mask);
  }
}

bool RatingLabel::on_button_press_event (GdkEventButton* e)
{
  if(e->button == 1) {
    int new_rating =  rating_value_from_hit_x(e->x);
    DBG_OUT("new rating = %d", new_rating);

    if(new_rating != m_rating) {
      set_rating(new_rating);
      signal_changed.emit(new_rating);
    }
  }
  return true;
}


void RatingLabel::get_preferred_width_vfunc (int& minimum_width, int& natural_width) const
{
  const Cairo::RefPtr<Cairo::ImageSurface> & star = get_star();
  minimum_width = natural_width = star->get_width() * 5;
}

void RatingLabel::get_preferred_height_vfunc (int& minimum_height, int& natural_height) const
{
  const Cairo::RefPtr<Cairo::ImageSurface> & star = get_star();
  minimum_height = natural_height = star->get_height();
}

bool RatingLabel::on_draw(const Cairo::RefPtr< Cairo::Context > &cr)
{
  const Cairo::RefPtr<Cairo::ImageSurface> & star = get_star();
  double x, y;
  x = 0;
  y = star->get_height();
  draw_rating(cr , m_rating, star, get_unstar(), x, y);
  return true;
}

void RatingLabel::on_state_changed (Gtk::StateType previous_state)
{
  Gtk::Widget::on_state_changed(previous_state);
}


}
