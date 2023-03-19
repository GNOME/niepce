/*
 * niepce - bin/rtengine.rs
 *
 * Copyright (C) 2023 Hubert Figuière
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

use rtengine::RtEngine;

fn main() {
    let mut args = std::env::args();
    if args.len() < 2 {
        println!("Filename is needed");
        std::process::exit(1);
    }
    if args.len() > 2 {
        println!("Ignoring extra arguments");
    }
    args.next();
    let filename = args.next().expect("Expect an argument");

    let engine = RtEngine::new();
    if engine.set_file(filename, true /* is_raw */).is_err() {
        std::process::exit(3);
    }

    match engine.process() {
        Err(error) => {
            println!("Error, couldn't render image: {error}");
            std::process::exit(2);
        }
        Ok(image) => {
            image.save_png("image.png").expect("Couldn't save image");
        }
    }
}
