#![allow(unused_imports)]

pub use self::inner::*;

#[macro_use]
pub mod inner {
    pub use colored::Colorize;
    pub use std::{
        format, println,
        string::{String, ToString},
        sync::atomic::{AtomicUsize, Ordering},
        time::Instant,
    };

    pub static NUM_INDENT: AtomicUsize = AtomicUsize::new(0);
    pub const PAD_CHAR: &str = "·";

    pub struct TimerInfo {
        pub msg: String,
        pub time: Instant,
    }

    #[macro_export]
    macro_rules! start_timer {
        ($msg:expr) => {{
            use $crate::ark_trace::inner::{
                AtomicUsize, Colorize, Instant, NUM_INDENT, Ordering, PAD_CHAR, ToString,
                compute_indent,
            };

            let msg = $msg();
            let start_info = "Start:".yellow().bold();
            let indent_amount = 2 * NUM_INDENT.fetch_add(0, Ordering::Relaxed);
            let indent = compute_indent(indent_amount);

            $crate::ark_trace::println!("{}{:8} {}", indent, start_info, msg);
            NUM_INDENT.fetch_add(1, Ordering::Relaxed);
            $crate::ark_trace::TimerInfo { msg: msg.to_string(), time: Instant::now() }
        }};
    }

    #[macro_export]
    macro_rules! end_timer {
        ($time:expr) => {{
            end_timer!($time, || "");
        }};
        ($time:expr, $msg:expr) => {{
            use $crate::ark_trace::inner::{
                AtomicUsize, Colorize, Instant, NUM_INDENT, Ordering, PAD_CHAR, ToString,
                compute_indent, format,
            };

            let time = $time.time;
            let final_time = time.elapsed();
            let final_time = {
                let secs = final_time.as_secs();
                let millis = final_time.subsec_millis();
                let micros = final_time.subsec_micros() % 1000;
                let nanos = final_time.subsec_nanos() % 1000;
                if secs != 0 {
                    format!("{}.{:03}s", secs, millis).bold()
                } else if millis > 0 {
                    format!("{}.{:03}ms", millis, micros).bold()
                } else if micros > 0 {
                    format!("{}.{:03}µs", micros, nanos).bold()
                } else {
                    format!("{}ns", final_time.subsec_nanos()).bold()
                }
            };

            let end_info = "End:".green().bold();
            let message = format!("{} {}", $time.msg, $msg());

            NUM_INDENT.fetch_sub(1, Ordering::Relaxed);
            let indent_amount = 2 * NUM_INDENT.fetch_add(0, Ordering::Relaxed);
            let indent = compute_indent(indent_amount);

            // Todo: Recursively ensure that *entire* string is of appropriate
            // width (not just message).
            $crate::ark_trace::println!(
                "{}{:8} {:.<pad$}{}",
                indent,
                end_info,
                message,
                final_time,
                pad = 75 - indent_amount
            );
        }};
    }

    #[macro_export]
    macro_rules! add_to_trace {
        ($title:expr, $msg:expr) => {{
            use $crate::ark_trace::{
                AtomicUsize, Colorize, Instant, NUM_INDENT, Ordering, PAD_CHAR, ToString,
                compute_indent, compute_indent_whitespace, format,
            };

            let start_msg = "StartMsg".yellow().bold();
            let end_msg = "EndMsg".green().bold();
            let title = $title();
            let start_msg = format!("{}: {}", start_msg, title);
            let end_msg = format!("{}: {}", end_msg, title);

            let start_indent_amount = 2 * NUM_INDENT.fetch_add(0, Ordering::Relaxed);
            let start_indent = compute_indent(start_indent_amount);

            let msg_indent_amount = 2 * NUM_INDENT.fetch_add(0, Ordering::Relaxed) + 2;
            let msg_indent = compute_indent_whitespace(msg_indent_amount);
            let mut final_message = "\n".to_string();
            for line in $msg().lines() {
                final_message += &format!("{}{}\n", msg_indent, line,);
            }

            // Todo: Recursively ensure that *entire* string is of appropriate
            // width (not just message).
            $crate::ark_trace::println!("{}{}", start_indent, start_msg);
            $crate::ark_trace::println!("{}{}", msg_indent, final_message,);
            $crate::ark_trace::println!("{}{}", start_indent, end_msg);
        }};
    }

    #[macro_export]
    macro_rules! add_single_trace {
        ($title:expr) => {{
            use $crate::ark_trace::{
                AtomicUsize, Colorize, Instant, NUM_INDENT, Ordering, PAD_CHAR, ToString,
                compute_indent, compute_indent_whitespace, format,
            };

            let start_msg = "Trace".blue().bold();
            let title = $title();
            let start_msg = format!("{}:   {}", start_msg, title);

            let indent_amount = 2 * NUM_INDENT.fetch_add(0, Ordering::Relaxed);
            let indent = compute_indent(indent_amount);

            // Todo: Recursively ensure that *entire* string is of appropriate
            // width (not just message).
            $crate::ark_trace::println!("{}{}", indent, start_msg);
        }};
    }

    pub fn compute_indent_whitespace(indent_amount: usize) -> String {
        let mut indent = String::new();
        for _ in 0..indent_amount {
            indent.push_str(" ");
        }
        indent
    }

    pub fn compute_indent(indent_amount: usize) -> String {
        let mut indent = String::new();
        for _ in 0..indent_amount {
            indent.push_str(&PAD_CHAR.white());
        }
        indent
    }
}

mod tests {
    use super::*;

    #[test]
    fn print_start_end() {
        let start = start_timer!(|| "Hello");
        end_timer!(start);
    }

    #[test]
    fn print_add() {
        add_single_trace!(|| "Hello");
        let start = start_timer!(|| "Hello");
        add_single_trace!(|| "HelloWorld");
        add_to_trace!(|| "HelloMsg", || "Hello, I\nAm\nA\nMessage");
        end_timer!(start);
    }
}
