use core::fmt::Write;

use edge_proto::EdgeCallReq;

use super::with_edge_caller;

pub struct EdgeConsole;

fn print_buffer_once(msg: &[u8]) {
    with_edge_caller(|caller| {
        caller
            .write_header(&EdgeCallReq::Print {
                len: msg.len() as u64,
            })
            .unwrap();
        caller.write_data(msg).unwrap();
        caller.kick().unwrap();
        assert!(caller.read_header().unwrap().is_ok());
    });
}

pub fn print_str(msg: &str) {
    for chunk in msg.as_bytes().chunks(super::EDGE_BUFFER_SIZE) {
        print_buffer_once(chunk);
    }
}

impl Write for EdgeConsole {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        print_str(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($args:tt)+) => ({
        use core::fmt::Write;
        write!($crate::edge::EdgeConsole, $($args)+).unwrap()
    });
}

#[macro_export]
macro_rules! println {
    () => ({
        $crate::print!("\n")
    });
    ($fmt:expr) => ({
        $crate::print!(concat!($fmt, "\n"))
    });
    ($fmt:expr, $($args:tt)+) => ({
        $crate::print!(concat!($fmt, "\n"), $($args)+)
    });
}
