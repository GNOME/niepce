/*
 * niepce - library/opqueue.h
 *
 * Copyright (C) 2007 Hubert Figuiere
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



#ifndef __NIEPCE_LIBRARY_OPQUEUE_H__
#define __NIEPCE_LIBRARY_OPQUEUE_H__

#include <deque>
#include <boost/thread/recursive_mutex.hpp>

#include "op.h"

namespace library {

	class OpQueue
	{
	public:
		OpQueue();
		~OpQueue();

		void add(const Op::Ptr &);
		Op::Ptr pop();
		bool isEmpty() const;

	private:
		std::deque<Op::Ptr> m_queue;
		typedef boost::recursive_mutex mutex_t;
		mutable mutex_t     m_mutex;
	};

}

#endif