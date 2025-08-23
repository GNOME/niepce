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
            print!("TRACE: ");
            println!( $($x),* );
        }
    };
}

/// Print a debug message.
#[macro_export]
macro_rules! dbg_out {
    ( $( $x:expr ),* $(,)?) => {
        {
            print!("DEBUG: ");
            println!( $($x),* );
        }
    };
}

/// Print an error message on the console.
#[macro_export]
macro_rules! err_out {
    ( $( $x:expr ),* $(,)?) => {
        {
            print!("ERROR: ");
            println!( $($x),* );
        }
    };
}

/// Like err_out!() but print the file and line number
#[macro_export]
macro_rules! err_out_line {
    ( $( $x:expr ),* $(,)?) => {
        {
            print!("ERROR: {}:{}:", file!(), line!());
            println!( $($x),* );
        }
    };
}

/// Assert and print a message if true.
/// Does NOT abort of call assert!()
#[macro_export]
macro_rules! dbg_assert {
    ( $cond:expr,  $msg:expr ) => {{
        if !$cond {
            print!("ASSERT: {}:{}: {}", file!(), line!(), stringify!($cond));
            println!($msg);
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
