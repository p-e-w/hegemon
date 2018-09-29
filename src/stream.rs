// Hegemon - A modular system monitor
// Copyright (C) 2018  Philipp Emanuel Weidmann <pew@worldwidemann.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use termion::color::Fg;

use theme::Theme;
use view::{format_quantity, printed_width};

pub trait StreamProvider {
    /// Returns a list of data stream objects.
    fn streams(&self) -> Vec<Box<Stream>>;
}

pub trait Stream {
    /// Returns the name of this data stream, to be used both as an identifier
    /// and for labeling the stream in the user interface.
    /// This method **must** return the same value each time it is called,
    /// and that value **must** be unique among all data streams.
    fn name(&self) -> String;

    /// Returns a detailed description of this data stream.
    /// This method **must** return the same value each time it is called.
    fn description(&self) -> String;

    /// Returns the current value of the quantity represented by this data stream,
    /// or `None` if no value can be determined at this time.
    fn value(&mut self) -> Option<f64>;

    /// Returns the minimum value of the quantity represented by this data stream,
    /// or `None` to have the minimum dynamically calculated from all value samples.
    /// This method **must** return the same value each time it is called.
    fn min(&self) -> Option<f64> {
        None
    }

    /// Returns the maximum value of the quantity represented by this data stream,
    /// or `None` to have the maximum dynamically calculated from all value samples.
    /// This method **must** return the same value each time it is called.
    fn max(&self) -> Option<f64> {
        None
    }

    /// Returns a human-readable representation of the given value.
    /// The result should make use of the appropriate colors from the given theme.
    fn format(&self, value: f64, theme: &Theme) -> String;

    /// Returns the maximum width, in characters when printed to the terminal,
    /// of all values that the `format` method can return.
    /// This method **must** return the same value each time it is called.
    fn format_width(&self) -> usize;
}

impl Stream {
    #[cfg_attr(feature = "cargo-clippy", allow(too_many_arguments))]
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        value: impl FnMut() -> Option<f64> + 'static,
        min: Option<f64>,
        max: Option<f64>,
        unit: impl Into<String>,
        digits_before_decimal: Option<usize>,
        precision: usize,
        signed: bool,
    ) -> Box<Stream> {
        let unit_1 = unit.into();
        let unit_2 = unit_1.clone();

        let use_prefix = digits_before_decimal.is_none();

        Box::new(SimpleStream {
            name: name.into(),
            description: description.into(),
            value: Box::new(value),
            min,
            max,
            format: Box::new(move |value: f64, theme: &Theme| {
                format_quantity(
                    value,
                    &unit_1,
                    use_prefix,
                    precision,
                    Fg(theme.stream_number_color),
                    Fg(theme.stream_unit_color),
                )
            }),
            format_width:
                // Sign
                (if signed { 1 } else { 0 }) +
                // Digits before decimal point
                digits_before_decimal.unwrap_or(3) +
                // Decimal point and digits after it
                (if precision > 0 { 1 + precision } else { 0 }) +
                // Unit prefix
                (if use_prefix { 1 } else { 0 }) +
                // Unit
                printed_width(unit_2),
        })
    }
}

struct SimpleStream {
    name: String,
    description: String,
    value: Box<FnMut() -> Option<f64>>,
    min: Option<f64>,
    max: Option<f64>,
    format: Box<Fn(f64, &Theme) -> String>,
    format_width: usize,
}

impl Stream for SimpleStream {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn description(&self) -> String {
        self.description.clone()
    }

    fn value(&mut self) -> Option<f64> {
        (self.value)()
    }

    fn min(&self) -> Option<f64> {
        self.min
    }

    fn max(&self) -> Option<f64> {
        self.max
    }

    fn format(&self, value: f64, theme: &Theme) -> String {
        (self.format)(value, theme)
    }

    fn format_width(&self) -> usize {
        self.format_width
    }
}
