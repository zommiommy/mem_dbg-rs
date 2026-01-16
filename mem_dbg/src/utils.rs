/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

/// Given a size in bytes, returns it in a human readable format using SI suffixes.
///
/// # Arguments
///
/// * `x` - The size to humanize.
///
/// # Examples
///
/// ```rust
/// use mem_dbg::humanize_float;
///
/// let (x, uom) = humanize_float(100);
/// assert_eq!(x, 100.0);
/// assert_eq!(uom, " B");
///
/// let (x, uom) = humanize_float(1234);
/// assert_eq!(x, 1.234);
/// assert_eq!(uom, "kB");
///
/// let (x, uom) = humanize_float(0);
/// assert_eq!(x, 0.0);
/// assert_eq!(uom, " B");
/// 
/// let (x, uom) = humanize_float(usize::MAX);
/// assert!(x > 1.0);
/// assert_eq!(uom, "EB");
/// ```
pub fn humanize_float(x: usize) -> (f64, &'static str) {
    const UOM: &[&str] = &[
        " B", "kB", "MB", "GB", "TB", "PB", "EB",
    ];
    let mut uom_idx = 0;
    debug_assert_eq!(UOM[uom_idx], " B");

    if x == 0 {
        return (0.0, UOM[uom_idx]);
    }

    let mut x = x as f64;

    while x >= 1000.0 && uom_idx < UOM.len() - 1 {
        uom_idx += 1;
        x /= 1000.0;
    }

    (x, UOM[uom_idx])
}

/// Returns the color code corresponding to the size.
///
/// # Arguments
///
/// * `x` - The size in bytes.
///
/// # Examples
///
/// ```rust
/// use mem_dbg::color;
/// use mem_dbg::reset_color;
///
/// assert_eq!(color(100), reset_color());
/// assert_eq!(color(1024), "\x1B[32m");
/// assert_eq!(color(1024 * 1024), "\x1B[33m");
/// assert_eq!(color(1024 * 1024 * 1024), "\x1B[31m");
/// ```
pub fn color(x: usize) -> &'static str {
    const KB: usize = 1024;
    const MB: usize = KB * KB;
    const GB: usize = MB * KB;
    #[allow(clippy::match_overlapping_arm)]
    match x {
        // white
        ..KB => reset_color(),
        // green
        ..MB => "\x1B[32m",
        // yellow
        ..GB => "\x1B[33m",
        // red
        _ => "\x1B[31m",
    }
}

/// Returns the color used to print types.
///
/// # Examples
///
/// ```rust
/// use mem_dbg::type_color;
///
/// assert_eq!(type_color(), "\x1B[38;2;128;128;128m");
/// ```
pub fn type_color() -> &'static str {
    // custom grey
    "\x1B[38;2;128;128;128m"
}

/// Returns the color code to reset the color.
///
/// # Examples
///
/// ```rust
/// use mem_dbg::reset_color;
///
/// assert_eq!(reset_color(), "\x1B[0m");
/// ```
pub fn reset_color() -> &'static str {
    "\x1B[0m"
}

/// Returns the number of digits of a number.
///
/// ```
/// use mem_dbg::n_of_digits;
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
