/*
 * niepce - framework/controller.h
 *
 * Copyright (C) 2007-2013 Hubert Figuiere
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



#ifndef __FRAMEWORK_CONTROLLER_H__
#define __FRAMEWORK_CONTROLLER_H__


#include <list>
#include <memory>

#include <sigc++/trackable.h>

#include "fwk/utils/databinder.hpp"


namespace fwk {

/** Generic controller class
 */
class Controller
    : public std::enable_shared_from_this<Controller>
    , public sigc::trackable
{
public:
    typedef std::shared_ptr<Controller> Ptr;
    typedef std::weak_ptr<Controller> WeakPtr;

    Controller();
    virtual ~Controller();

    /** add a subcontroller to this one */
    void add(const Ptr & sub);
    void remove(const Ptr & sub);

    virtual bool canTerminate();
    /** signal that the controller needs to terminate */
    virtual void terminate();

    /** called when everything is ready
     * subclasses should reimplement if needed
     */
    virtual void on_ready();
protected:
    /** called when the controller has been added to a parent. */
    virtual void _added();

    void _ready();

    std::list<Ptr> m_subs; /**< sub controllers */

    DataBinderPool m_databinders;
};

}

/*
  Local Variables:
  mode:c++
  c-file-style:"stroustrup"
  c-file-offsets:((innamespace . 0))
  indent-tabs-mode:nil
  fill-column:99
  End:
*/

#endif
