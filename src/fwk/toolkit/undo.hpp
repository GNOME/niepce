/*
 * niepce - fwk/toolkit/undo.hpp
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

#include <functional>

namespace fwk {

class UndoListener {
public:
  typedef std::function<void ()> function_t;

  UndoListener(function_t&& f)
    : m_f(f)
  {}
  void call() const
  {
    m_f();
  }

private:
  function_t m_f;
};

template<class T>
class UndoFn {
public:
    typedef std::function<void (T)> function_t;

    UndoFn(function_t&& f)
        : m_f(f)
        {}
    void call(T v) const
        {
            m_f(v);
        }

private:
    function_t m_f;
};

template<>
class UndoFn<void> {
public:
    typedef std::function<void ()> function_t;

    UndoFn<void>(function_t&& f)
        : m_f(f)
        {}
    void call() const
        {
            m_f();
        }

private:
    function_t m_f;
};

template<class T>
class RedoFn {
public:
  typedef std::function<T ()> function_t;

  RedoFn(function_t&& f)
    : m_f(f)
  {}
  T call() const
  {
    return m_f();
  }

private:
  function_t m_f;
};

typedef RedoFn<int64_t> RedoFnInt;
typedef UndoFn<int64_t> UndoFnInt;
typedef RedoFn<void> RedoFnVoid;
typedef UndoFn<void> UndoFnVoid;
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
