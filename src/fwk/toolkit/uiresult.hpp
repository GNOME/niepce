/* -*- mode: C++; tab-width: 4; c-basic-offset: 4; indent-tabs-mode:nil; -*- */
/*
 * niepce - fwk/toolkit/uiresult.hpp
 *
 * Copyright (C) 2017-2022 Hubert Figuière
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

#include <deque>
#include <mutex>

#include <glibmm/dispatcher.h>

#include "fwk/base/option.hpp"

namespace fwk {

/** @brief Fetch a "result" asynchronously */
class UIResult
{
public:
    virtual void clear() = 0;

    sigc::connection connect(sigc::slot<void()> slot) {
        return m_notifier.connect(std::move(slot));
    }

    void run(std::function<void()>&& f);
protected:
    Glib::Dispatcher m_notifier;
    std::mutex m_data_mutex;
};

template<class T>
class UIResultSingle
    : public UIResult
{
public:
    void clear() override {
        m_data = T();
    }

    void send_data(T&& d) {
        {
            std::lock_guard<std::mutex> lock(m_data_mutex);
            m_data = std::move(d);
        }
        m_notifier.emit();
    }
    T recv_data() {
        std::lock_guard<std::mutex> lock(m_data_mutex);
        return m_data;
    }
private:
  T m_data;
};

/** @brief Fetch many "results" asynchronously */
template<class T>
class UIResults
    : public UIResult
{
public:
    void clear() override {
        m_data.clear();
    }

    void send_data(T&& d) {
        {
            std::lock_guard<std::mutex> lock(m_data_mutex);
            m_data.push_back(std::move(d));
        }
        m_notifier.emit();
    }
    Option<T> recv_data() {
        std::lock_guard<std::mutex> lock(m_data_mutex);
        if (m_data.empty()) {
            return Option<T>();
        }
        auto result = Option<T>(m_data.front());
        m_data.pop_front();
        return result;
    }
private:
    std::deque<T> m_data;
};

}
