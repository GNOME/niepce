/*
 * niepce - fwk/base/debug.rs
 *
 * Copyright (C) 2019-2026 Hubert Figuière
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

/// Use this macro to output a message on a Result returning an error.
/// This allow removing the warning if you ignore the Result.
/// `Error` must implement `Debug`
#[macro_export]
macro_rules! on_err_out {
    ($e:expr) => {
        if let Err(ref err) = $e {
            $crate::err_out!(
                "{}:{} Error '{}': {:?}",
                file!(),
                line!(),
                stringify!($e),
                err
            );
        }
    };
}

/// Print a trace message.
#[macro_export]
macro_rules! trace_out {
    ( $( $x:expr ),* $(,)?) => {
        {
            $crate::log::trace!( $($x),* );
        }
    };
}

/// Print a debug message.
#[macro_export]
macro_rules! dbg_out {
    ( $( $x:expr ),* $(,)?) => {
        {
            $crate::log::debug!( $($x),* );
        }
    };
}

/// Print an error message on the console.
#[macro_export]
macro_rules! err_out {
    ( $( $x:expr ),* $(,)?) => {
        {
            $crate::log::error!( $($x),* );
        }
    };
}

/// Like err_out!() but print the file and line number
#[macro_export]
macro_rules! err_out_line {
    ( $( $x:expr ),* $(,)?) => {
        {
            let message = format!( $($x),* );
            $crate::log::error!("{}:{}: {}", file!(), line!(), message);
        }
    };
}

/// Assert and print a message if true.
/// Does NOT abort of call assert!()
#[macro_export]
macro_rules! dbg_assert {
    ( $cond:expr,  $msg:expr ) => {{
        if !$cond {
            let message = format!("ASSERT: {}:{}: {}", file!(), line!(), stringify!($cond));
            $crate::log::error!("{} {}", message, $msg);
        }
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        dbg_out!("debug {}", 42);
        err_out!("error {}", 69);
        trace_out!("trace {}", 666);
        dbg_assert!(false, "failed assert");
    }
}
