/*
 * niepce - framework/configuration.h
 *
 * Copyright (C) 2007 Hubert Figuiere
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  
 * 02110-1301, USA
 */


#include "configuration.h"


namespace framework {

	const std::string & Configuration::operator[](const std::string & key) const
	{
		static std::string empty;

		config_map_t::const_iterator iter = m_configdata.find(key);
		if(iter == m_configdata.end()) {
			return empty;
		}
		
		return iter->second;
	}

}
