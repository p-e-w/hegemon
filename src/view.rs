// Hegemon - A modular system monitor
// Copyright (C) 2018-2020  Philipp Emanuel Weidmann <pew@worldwidemann.com>
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

use std::cmp::max;
use std::f64;
use std::fmt::Display;
use std::time::Duration;

use regex::Regex;
use termion::color::{Bg, Fg};
use termion::cursor;
use termion::style::Reset;

use crate::model::{Application, MenuItem, Screen, ScrollAnchor, StreamWrapper};
use crate::theme::Theme;

const EXPANDED_GRAPH_HEIGHT: usize = 5;

const STATS_LABEL: &str = "lo/hi/avg";

const DOT: &str = "\u{2022}";
const BARS: &[&str] = &[
    "\u{2581}", "\u{2582}", "\u{2583}", "\u{2584}", "\u{2585}", "\u{2586}", "\u{2587}", "\u{2588}",
];

impl Application {
    pub fn render(&self, theme: &Theme) -> String {
        let mut string = format!("{}{}", cursor::Goto(1, 1), Reset);

        match self.screen {
            Screen::Main => {
                let name_width = self.name_width();
                let value_width = self.value_width();

                let width = max(self.width, name_width + 3 + value_width);
                let height = max(self.height, 3);

                let graph_width = width - name_width - value_width - 2;

                let interval = self.interval();
                let full_intervals = (graph_width - 1) / interval.tick_spacing;
                let first_tick_padding = name_width + 1 + (graph_width - 1 - (full_intervals * interval.tick_spacing));

                // Render top bar
                string.push_str(&format!("{}", Bg(theme.top_bar_color)));
                string.push_str(&" ".repeat(first_tick_padding));

                for i in (1..=full_intervals).rev() {
                    let time = interval.duration * ((i * interval.tick_spacing) as u32);
                    let time_string =
                        format_duration(time, Fg(theme.top_bar_number_color), Fg(theme.top_bar_unit_color));
                    string.push_str(&pad_right(time_string, interval.tick_spacing));
                }

                string.push_str(&format!("{}Now", Fg(theme.top_bar_unit_color)));
                string.push_str(&" ".repeat(value_width - 1));

                let max_lines = height - 2;

                let streams = self.active_streams();

                let indices = match self.scroll_anchor {
                    ScrollAnchor::Top => (self.scroll_index..streams.len()).collect::<Vec<_>>(),
                    ScrollAnchor::Bottom => (0..=self.scroll_index).rev().collect::<Vec<_>>(),
                };

                let mut lines = Vec::new();

                // Render data streams
                'outer: for i in indices {
                    let mut stream_lines = streams[i].render(
                        i,
                        i == self.selection_index,
                        name_width,
                        graph_width,
                        value_width,
                        interval.tick_spacing,
                        theme,
                    );

                    if self.scroll_anchor == ScrollAnchor::Bottom {
                        stream_lines.reverse();
                    }

                    for line in stream_lines {
                        if lines.len() >= max_lines {
                            break 'outer;
                        }

                        match self.scroll_anchor {
                            ScrollAnchor::Top => lines.push(line),
                            ScrollAnchor::Bottom => lines.insert(0, line),
                        }
                    }
                }

                if !lines.is_empty() {
                    string.push_str("\n\r");
                    string.push_str(&lines.join("\n\r"));
                }

                // Render empty lines below data streams
                if lines.len() < max_lines {
                    let background_color = if streams.len() % 2 == 0 {
                        theme.stream_even_background_color
                    } else {
                        theme.stream_odd_background_color
                    };

                    let tick = format!("{} {}", Bg(theme.tick_color), Bg(background_color));

                    let empty_line = format!(
                        "\n\r{}{}{}{}{}",
                        Bg(background_color),
                        " ".repeat(first_tick_padding),
                        format!("{}{}", tick, " ".repeat(interval.tick_spacing - 1)).repeat(full_intervals),
                        tick,
                        " ".repeat(1 + value_width),
                    );

                    string.push_str(&empty_line.repeat(max_lines - lines.len()));
                }
            }

            Screen::Streams => {
                let message = ellipsize("Stream selection is not implemented yet", self.width);

                string.push_str(&format!(
                    "{}{}{}",
                    Fg(theme.stream_name_color),
                    Bg(theme.stream_odd_background_color),
                    pad_right(message, self.width),
                ));

                string.push_str(&format!("\n\r{}", " ".repeat(self.width)).repeat(max(self.height, 2) - 2));
            }
        }

        // Render bottom bar
        let (left_menu, right_menu) = self.menu();
        let left_menu_string = left_menu.iter().map(|m| m.render(theme)).collect::<Vec<_>>().join("  ");
        let right_menu_string = right_menu
            .iter()
            .map(|m| m.render(theme))
            .collect::<Vec<_>>()
            .join("  ");

        let mut menu_width = printed_width(&left_menu_string) + 2 + printed_width(&right_menu_string) + 1;

        string.push_str(&format!("\n\r{}", left_menu_string));
        if self.screen == Screen::Main {
            let interval_string = format_duration(
                self.interval().duration,
                Fg(theme.bottom_bar_number_color),
                Fg(theme.bottom_bar_unit_color),
            );
            string.push_str(&format!(" {}", interval_string));
            menu_width += 1 + printed_width(&interval_string);
        }

        string.push_str("  ");
        if menu_width < self.width {
            string.push_str(&" ".repeat(self.width - menu_width));
        }

        string.push_str(&format!("{} {}", right_menu_string, Reset));

        string
    }

    fn name_width(&self) -> usize {
        self.active_streams()
            .iter()
            .map(|s| {
                max(
                    // Any name ...
                    printed_width(s.stream.name()),
                    // ... and any value must fit, because the name column
                    // also displays values (tick labels) for expanded streams
                    s.stream.format_width(),
                )
            })
            .max()
            .unwrap_or(0)
    }

    fn value_width(&self) -> usize {
        max(
            // Any value ...
            self.active_streams()
                .iter()
                .map(|s| s.stream.format_width())
                .max()
                .unwrap_or(0),
            // ... and the stats label must fit
            printed_width(STATS_LABEL),
        )
    }
}

impl StreamWrapper {
    #[allow(clippy::too_many_arguments)]
    fn render(
        &self,
        index: usize,
        selected: bool,
        name_width: usize,
        graph_width: usize,
        value_width: usize,
        tick_spacing: usize,
        theme: &Theme,
    ) -> Vec<String> {
        let mut lines = Vec::new();

        let graph_color = theme.stream_graph_colors[index % theme.stream_graph_colors.len()];

        let background_color = if selected {
            theme.stream_selected_background_color
        } else if index % 2 == 0 {
            theme.stream_even_background_color
        } else {
            theme.stream_odd_background_color
        };

        let graph = |values: Vec<Option<f64>>, min: f64, max: f64| {
            let mut graph = format!("{}{}", Fg(graph_color.0), Bg(background_color));

            for (i, value) in values.iter().enumerate() {
                let symbol = match value {
                    Some(number) => {
                        let bar_index = if min < max {
                            let level = (number - min) / (max - min);
                            let bucket = (level * (BARS.len() as f64)).ceil() as usize;
                            if bucket == 0 {
                                0
                            } else {
                                bucket - 1
                            }
                        } else {
                            0
                        };
                        BARS[bar_index]
                    }
                    None => DOT,
                };

                if ((graph_width - 1) - i) % tick_spacing == 0 {
                    // Tick intersection
                    graph.push_str(&format!(
                        "{}{}{}{}{}",
                        Fg(graph_color.1),
                        Bg(theme.tick_color),
                        symbol,
                        Fg(graph_color.0),
                        Bg(background_color),
                    ));
                } else {
                    graph.push_str(symbol);
                }
            }

            graph
        };

        let values = (1..=graph_width)
            .rev()
            .map(|i| {
                if i <= self.values.len() {
                    self.values[self.values.len() - i]
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let numbers = values.iter().cloned().filter_map(|v| v).collect::<Vec<_>>();

        let numbers_min = numbers.iter().cloned().fold(f64::NAN, f64::min);
        let numbers_max = numbers.iter().cloned().fold(f64::NAN, f64::max);
        let numbers_avg = numbers.iter().cloned().sum::<f64>() / (numbers.len() as f64);

        let min = self.stream.min().unwrap_or(numbers_min);
        let max = self.stream.max().unwrap_or(numbers_max);

        let value_string = if numbers.is_empty() {
            String::new()
        } else {
            self.stream.format(numbers[numbers.len() - 1], theme)
        };

        let mut line = format!(
            "{}{}",
            Fg(if selected {
                theme.stream_name_selected_text_color
            } else {
                theme.stream_name_color
            }),
            Bg(if selected {
                theme.stream_name_selected_background_color
            } else {
                background_color
            }),
        );

        line.push_str(&pad_left(self.stream.name(), name_width));
        line.push_str(&format!("{} ", Bg(background_color)));

        if self.expanded {
            line.push_str(&format!(
                "{}{} {}",
                Fg(theme.stream_description_color),
                pad_right(ellipsize(self.stream.description(), graph_width), graph_width),
                pad_right(value_string, value_width),
            ));

            lines.push(line);

            let mut graph_rows = Vec::new();

            for i in (0..EXPANDED_GRAPH_HEIGHT).rev() {
                let row_height = (max - min) / (EXPANDED_GRAPH_HEIGHT as f64);
                let row_min = min + (row_height * (i as f64));
                let row_max = row_min + row_height;

                let row_values = values
                    .iter()
                    .map(|v| {
                        v.and_then(|number| {
                            if number < row_min {
                                None
                            } else if number > row_max {
                                Some(row_max)
                            } else {
                                Some(number)
                            }
                        })
                    })
                    .collect::<Vec<_>>();

                graph_rows.push(graph(row_values, row_min, row_max));
            }

            let (min_string, mid_string, max_string) = if min.is_finite() && max.is_finite() {
                (
                    self.stream.format(min, theme),
                    self.stream.format((min + max) / 2.0, theme),
                    self.stream.format(max, theme),
                )
            } else {
                (String::new(), String::new(), String::new())
            };

            let (stats_label, low_string, high_string, avg_string) =
                if numbers_min.is_finite() && numbers_max.is_finite() && numbers_avg.is_finite() {
                    (
                        String::from(STATS_LABEL),
                        self.stream.format(numbers_min, theme),
                        self.stream.format(numbers_max, theme),
                        self.stream.format(numbers_avg, theme),
                    )
                } else {
                    (String::new(), String::new(), String::new(), String::new())
                };

            let y_mid = EXPANDED_GRAPH_HEIGHT / 2;
            let y_max = EXPANDED_GRAPH_HEIGHT - 1;

            for (y, row) in graph_rows.iter().enumerate() {
                let left_axis = if y == 0 {
                    &max_string
                } else if y == y_mid {
                    &mid_string
                } else if y == y_max {
                    &min_string
                } else {
                    ""
                };
                let right_axis = if y == y_max - 3 {
                    &stats_label
                } else if y == y_max - 2 {
                    &low_string
                } else if y == y_max - 1 {
                    &high_string
                } else if y == y_max {
                    &avg_string
                } else {
                    ""
                };

                lines.push(format!(
                    "{}{} {} {}{}",
                    Bg(background_color),
                    pad_left(left_axis, name_width),
                    row,
                    Fg(theme.stream_description_color),
                    pad_right(right_axis, value_width)
                ));
            }
        } else {
            line.push_str(&format!(
                "{} {}",
                graph(values, min, max),
                pad_right(value_string, value_width),
            ));

            lines.push(line);
        }

        lines
    }

    pub fn height(&self) -> usize {
        if self.expanded {
            1 + EXPANDED_GRAPH_HEIGHT
        } else {
            1
        }
    }
}

impl MenuItem {
    fn render(&self, theme: &Theme) -> String {
        format!(
            "{}{}\u{2590}{}{}{}{}{}\u{258C}{}{}",
            Fg(theme.bottom_bar_key_background_color),
            Bg(theme.bottom_bar_color),
            Fg(theme.bottom_bar_key_text_color),
            Bg(theme.bottom_bar_key_background_color),
            self.keys,
            Fg(theme.bottom_bar_key_background_color),
            Bg(theme.bottom_bar_color),
            Fg(theme.bottom_bar_label_color),
            self.label,
        )
    }
}

pub fn format_quantity(
    quantity: f64,
    unit: impl Display,
    use_prefix: bool,
    precision: usize,
    number_style: impl Display,
    unit_style: impl Display,
) -> String {
    assert!(quantity.is_finite());

    let magnitude = if use_prefix && quantity != 0.0 {
        let m = (quantity.abs().log10() / 3.0).floor() as i32;

        if format!("{:.*}", precision, quantity / 10.0_f64.powi(3 * m)).starts_with("1000") {
            // Rounding will increase the apparent magnitude
            m + 1
        } else {
            m
        }
    } else {
        0
    };

    let prefix = if magnitude != 0 {
        let prefixes = if magnitude > 0 {
            vec!["k", "M", "G", "T", "P", "E"]
        } else {
            vec!["m", "\u{B5}", "n", "p", "f", "a"]
        };

        let index = (magnitude.abs() - 1) as usize;

        if index < prefixes.len() {
            prefixes[index]
        } else {
            "?"
        }
    } else {
        ""
    };

    let mut number = format!("{:.*}", precision, quantity / 10.0_f64.powi(3 * magnitude));

    if precision > 0 {
        // Remove trailing zeros
        let regex = Regex::new(r"\.?0+$").unwrap();
        number = regex.replace(&number, "").into_owned();
    }

    format!("{}{}{}{}{}", number_style, number, unit_style, prefix, unit)
}

fn format_duration(duration: Duration, number_style: impl Display, unit_style: impl Display) -> String {
    let mut milliseconds = duration.as_millis();

    if milliseconds == 0 {
        return format!("{}0{}s", number_style, unit_style);
    }

    let mut string = String::new();

    for (unit, factor) in &[("h", 3_600_000), ("m", 60_000)] {
        if milliseconds >= *factor {
            string.push_str(&format!(
                "{}{}{}{}",
                number_style,
                milliseconds / factor,
                unit_style,
                unit
            ));
            milliseconds %= factor;
        }
    }
    if milliseconds > 0 {
        string.push_str(&format!(
            "{}{:1}{}s",
            number_style,
            milliseconds as f64 / 1000.0,
            unit_style
        ));
    }

    string
}

pub fn printed_width(string: impl AsRef<str>) -> usize {
    // Matches ANSI SGR control sequences (text attributes),
    // which don't affect the printed width
    let regex = Regex::new(r"\x1B\[.*?m").unwrap();
    regex.replace_all(string.as_ref(), "").chars().count()
}

fn ellipsize(string: impl Into<String>, width: usize) -> String {
    assert!(width > 0);

    let s = string.into();

    if s.chars().count() > width {
        let truncated_string: String = s.chars().take(width - 1).collect();
        format!("{}\u{2026}", truncated_string)
    } else {
        s
    }
}

fn pad_left(string: impl AsRef<str>, width: usize) -> String {
    format!(
        "{}{}",
        " ".repeat(width - printed_width(string.as_ref())),
        string.as_ref(),
    )
}

fn pad_right(string: impl AsRef<str>, width: usize) -> String {
    format!(
        "{}{}",
        string.as_ref(),
        " ".repeat(width - printed_width(string.as_ref())),
    )
}

#[cfg(test)]
mod tests {
    use termion::{
        color::{Bg, Fg, Green, Red},
        style::Bold,
    };

    use super::*;

    #[test]
    fn test_format_quantity() {
        assert_eq!(format_quantity(0.0, "C", true, 0, "A", "B"), "A0BC");
        assert_eq!(format_quantity(0.001, "C", true, 0, "A", "B"), "A1BmC");
        assert_eq!(format_quantity(0.999, "C", true, 0, "A", "B"), "A999BmC");
        assert_eq!(format_quantity(1.0, "C", true, 0, "A", "B"), "A1BC");
        assert_eq!(format_quantity(999.0, "C", true, 0, "A", "B"), "A999BC");
        assert_eq!(format_quantity(1000.0, "C", true, 0, "A", "B"), "A1BkC");
        assert_eq!(format_quantity(0.9999, "C", true, 0, "A", "B"), "A1BC");
        assert_eq!(format_quantity(999.9, "C", true, 0, "A", "B"), "A1BkC");
        assert_eq!(format_quantity(999_900.0, "C", true, 0, "A", "B"), "A1BMC");
        assert_eq!(format_quantity(123_456_789.0, "C", true, 3, "A", "B"), "A123.457BMC");
        assert_eq!(format_quantity(123_456_789.0, "C", false, 3, "A", "B"), "A123456789BC");
        assert_eq!(
            format_quantity(-0.000_000_001_234_567_89, "C", true, 3, "A", "B"),
            "A-1.235BnC",
        );
        assert_eq!(
            format_quantity(-0.000_000_001_234_567_89, "C", false, 3, "A", "B"),
            "A-0BC",
        );
        assert_eq!(format_quantity(10.0_f64.powi(100), "C", true, 0, "A", "B"), "A10B?C");
        assert_eq!(format_quantity(10.0_f64.powi(-100), "C", true, 0, "A", "B"), "A100B?C");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(0), "A", "B"), "A0Bs");
        assert_eq!(format_duration(Duration::from_secs(1), "A", "B"), "A1Bs");
        assert_eq!(format_duration(Duration::from_secs(60), "A", "B"), "A1Bm");
        assert_eq!(format_duration(Duration::from_secs(61), "A", "B"), "A1BmA1Bs");
        assert_eq!(format_duration(Duration::from_secs(3600), "A", "B"), "A1Bh");
        assert_eq!(format_duration(Duration::from_secs(3660), "A", "B"), "A1BhA1Bm");
        assert_eq!(format_duration(Duration::from_secs(3601), "A", "B"), "A1BhA1Bs");
        assert_eq!(format_duration(Duration::from_secs(3661), "A", "B"), "A1BhA1BmA1Bs");
        assert_eq!(
            format_duration(Duration::from_secs(100_000), "A", "B"),
            "A27BhA46BmA40Bs",
        );
    }

    #[test]
    fn test_printed_width() {
        assert_eq!(printed_width(""), 0);
        assert_eq!(printed_width(" "), 1);
        assert_eq!(printed_width("\u{21}\u{2190}\u{1F800}"), 3);
        assert_eq!(printed_width(format!("{}{}{}", Fg(Red), Bg(Green), Bold)), 0);
        assert_eq!(printed_width(format!("ab{}cd{}ef{}gh", Fg(Red), Bg(Green), Bold)), 8);
        assert_eq!(
            printed_width(format!("ab\u{21}{}cd\u{2190}{}ef\u{1F800}{}", Fg(Red), Bg(Green), Bold)),
            9,
        );
    }
}
