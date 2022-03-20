/*
 * niepce - fwk/toolkit/widgets/tokentextview.cpp
 *
 * Copyright (C) 2012-2022 Hubert Figuière
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

#include <boost/algorithm/string.hpp>
#include <boost/algorithm/string/split.hpp>
#include "fwk/base/debug.hpp"
#include "fwk/utils/stringutils.hpp"
#include "tokentextview.hpp"

namespace fwk {

TokenTextView::TokenTextView()
    : NoTabTextView()
{
    set_wrap_mode(Gtk::WrapMode::WORD);
}

void TokenTextView::set_tokens(const Tokens & tokens)
{
    std::string v = fwk::join(tokens, ", ");
    get_buffer()->set_text(v);
}

void TokenTextView::get_tokens(Tokens & tokens)
{
    Glib::ustring t = get_buffer()->get_text();
    boost::split(tokens, t.raw(), boost::is_any_of(","));
    // trim the tokens from whitespace
    for_each(tokens.begin(), tokens.end(),
             [](std::string & token) {
                 boost::trim(token);
             });
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
