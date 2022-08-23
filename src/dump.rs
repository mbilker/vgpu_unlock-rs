// SPDX-License-Identifier: MIT

use std::cmp;
use std::fmt::Write;

#[allow(dead_code)]
pub fn dump(data: &[u8]) -> String {
    let mut output = String::new();

    if data.is_empty() {
        output.push_str("\t--- Empty ---");
    }

    for i in (0..data.len()).step_by(16) {
        let to_print = cmp::min(16, data.len() - i);
        let to_pad = 16 - to_print;
        let data = &data[i..i + to_print];

        let _ = write!(output, "    {:08x}", i);

        for byte in data {
            let _ = write!(output, " {:02x}", byte);
        }

        for _ in 0..to_pad {
            output.push_str("   ");
        }

        output.push(' ');
        output.extend(data.iter().map(|&c| {
            if !(0x20..0x7f).contains(&c) {
                '.'
            } else {
                c as char
            }
        }));
        output.push('\n');
    }

    output.push('\n');

    output
}
