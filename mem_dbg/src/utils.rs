/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

/// Given a float, return it in a human readable format using SI suffixes.
pub fn humanize_float(mut x: f64) -> (f64, &'static str) {
    const UOM: &[&str] = &[
        "qB", "rB", "yB", "zB", "aB", "fB", "pB", "nB", "μB", "mB", " B", "KB", "MB", "GB", "TB",
        "PB", "EB", "ZB", "YB", "RB", "QB",
    ];
    let mut uom_idx = 10;
    debug_assert_eq!(UOM[uom_idx], " B");

    if x == 0.0 {
        return (0.0, UOM[uom_idx]);
    }

    if x.abs() > 1.0 {
        while x.abs() > 1000.0 && uom_idx < UOM.len() - 1 {
            uom_idx += 1;
            x /= 1000.0;
        }
    } else {
        while x.abs() < 0.001 && uom_idx > 0 {
            uom_idx -= 1;
            x *= 1000.0;
        }
    }

    (x, UOM[uom_idx])
}

/// Return the number of digits of a number.
///
/// ```
/// use mem_dbg::utils::n_of_digits;
///
/// assert_eq!(n_of_digits(0), 1);
/// assert_eq!(n_of_digits(1), 1);
/// assert_eq!(n_of_digits(10), 2);
/// assert_eq!(n_of_digits(100), 3);
/// assert_eq!(n_of_digits(1000), 4);
/// assert_eq!(n_of_digits(10000), 5);
/// assert_eq!(n_of_digits(100000), 6);
/// ```
pub fn n_of_digits(x: usize) -> usize {
    if x == 0 {
        return 1;
    }
    let mut digits = 0;
    let mut x = x;
    while x > 0 {
        digits += 1;
        x /= 10;
    }
    digits
}
