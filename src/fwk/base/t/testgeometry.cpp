/*
 * niepce - utils/testgeometry.cpp
 *
 * Copyright (C) 2007-2023 Hubert Figuiere
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
/** @brief unit test for files */

#include <gtest/gtest.h>

#include <stdlib.h>
#include <vector>
#include "fwk/base/geometry.hpp"

using fwk::Rect;

TEST(TestGeometry, TestGeometrySanity)
{
	Rect r1(0,1,2,3);
	ASSERT_EQ(std::to_string(r1), "0 1 2 3");

	std::string s("100 100 250 250");
	Rect r2(s);
	ASSERT_EQ(r2.x(), 100);
	ASSERT_EQ(r2.y(), 100);
	ASSERT_EQ(r2.w(), 250);
	ASSERT_EQ(r2.h(), 250);
	std::vector<std::string> vtest;
	vtest.push_back("a b c d");
	vtest.push_back("100 100 150");
	std::for_each(vtest.begin(), vtest.end(),
		      [] (const std::string & value) {
			      bool raised = false;
			      try {
				      Rect r3(value);
			      }
			      catch(const std::bad_cast&) {
				      raised = true;
			      }
			      ASSERT_TRUE(raised);
		      }
		);

}
