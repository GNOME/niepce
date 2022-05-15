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

#include <list>
#include <stack>
#include <string>
#include <memory>

#include <sigc++/signal.h>
#include <sigc++/trackable.h>

#include "fwk/toolkit/command.hpp"
#include "fwk/base/util.hpp"

namespace fwk {


class UndoTransaction
{
public:
    NON_COPYABLE(UndoTransaction);

    UndoTransaction(const std::string & n);
    ~UndoTransaction();

    template <typename _RetType>
    std::shared_ptr<Command> new_command(
        const typename CommandWithArg<_RetType>::RedoFunction&,
        const typename CommandWithArg<_RetType>::UndoFunction&);
    void undo();
    void redo();
    /** execute the transaction after adding it. (calls %undo) */
    void execute()
        { redo(); }
    const std::string & name() const
        { return m_name; }
protected:
    /** add the command. Use %new_command instead */
    void add(const std::shared_ptr<Command>&);
private:
    std::list<std::shared_ptr<Command>> m_operations;
    std::string m_name;
};

template <typename _ArgType>
std::shared_ptr<Command> UndoTransaction::new_command(
    const typename CommandWithArg<_ArgType>::RedoFunction & _redo,
    const typename CommandWithArg<_ArgType>::UndoFunction & _undo)
{
    auto cmd = std::make_shared<CommandWithArg<_ArgType>>(_redo, _undo);
    add(cmd);
    return cmd;
}


class UndoHistory
    : public sigc::trackable
{
public:
    NON_COPYABLE(UndoHistory);

    UndoHistory() {}
    ~UndoHistory();

    /** the history becomes owner */
    void add(const std::shared_ptr<UndoTransaction>&);
    void undo();
    void redo();
    void clear();
    bool has_undo() const
        { return !m_undos.empty(); }
    bool has_redo() const
        { return !m_redos.empty(); }
    std::string next_undo() const;
    std::string next_redo() const;

    // called when the undo history change.
    sigc::signal<void(void)> signal_changed;
private:
    std::list<std::shared_ptr<UndoTransaction>> m_undos;
    std::list<std::shared_ptr<UndoTransaction>> m_redos;
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
