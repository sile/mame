use std::fmt::Write;

use crate::terminal::{UnicodeTerminalFrame, str_cols};

pub fn render_legend<I, T>(title: &str, items: I) -> UnicodeTerminalFrame
where
    I: Iterator<Item = T>,
    T: std::fmt::Display,
{
    let items = items.map(|x| x.to_string()).collect::<Vec<_>>();
    let rows = items.len() + 1; // 1 = "─"
    let cols = std::iter::once(title.len() + 4) // 4 = "└ " + " ─"
        .chain(items.iter().map(|x| str_cols(x) + 2)) // 2 = "│ "
        .max()
        .expect("infallible");

    let mut frame = UnicodeTerminalFrame::new(tuinix::TerminalSize::rows_cols(rows, cols));

    for item in &items {
        writeln!(frame, "│ {item} ").expect("infallible");
    }

    write!(frame, "└").expect("infallible");
    if title.is_empty() {
        for _ in 1..cols {
            write!(frame, "─").expect("infallible");
        }
    } else {
        let offset = (cols - (title.len() + 2)) / 2;
        for _ in 1..offset {
            write!(frame, "─").expect("infallible");
        }
        write!(frame, " {title} ").expect("infallible");
        for _ in (offset + title.len() + 2)..cols {
            write!(frame, "─").expect("infallible");
        }
    }
    writeln!(frame).expect("infallible");

    frame
}
