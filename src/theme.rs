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

use termion::color::AnsiValue;

pub struct Theme {
    /// Background color of the main screen's top bar
    pub top_bar_color: AnsiValue,
    /// Color of numbers in the top bar's interval tick labels
    pub top_bar_number_color: AnsiValue,
    /// Color of units in the top bar's interval tick labels
    pub top_bar_unit_color: AnsiValue,
    /// Color of vertical lines intersecting the graphs
    pub tick_color: AnsiValue,
    /// Background color of even-numbered streams (count starts at zero)
    pub stream_even_background_color: AnsiValue,
    /// Background color of odd-numbered streams (count starts at zero)
    pub stream_odd_background_color: AnsiValue,
    /// Background color of the selected stream
    pub stream_selected_background_color: AnsiValue,
    /// Color of stream names in unselected streams
    pub stream_name_color: AnsiValue,
    /// Foreground color of the selected stream's name
    pub stream_name_selected_text_color: AnsiValue,
    /// Background color of the selected stream's name
    pub stream_name_selected_background_color: AnsiValue,
    /// Color of stream descriptions
    pub stream_description_color: AnsiValue,
    /// Color of numbers in stream values
    pub stream_number_color: AnsiValue,
    /// Color of units in stream values
    pub stream_unit_color: AnsiValue,
    /// Colors of stream graphs, to be repeated cyclically.
    /// The first element in each pair is the regular color,
    /// the second the color for tick intersections.
    pub stream_graph_colors: Vec<(AnsiValue, AnsiValue)>,
    /// Background color of the bottom bar
    pub bottom_bar_color: AnsiValue,
    /// Foreground color of key labels in the bottom bar's menu items
    pub bottom_bar_key_text_color: AnsiValue,
    /// Background color of key labels in the bottom bar's menu items
    pub bottom_bar_key_background_color: AnsiValue,
    /// Color of labels in the bottom bar's menu items
    pub bottom_bar_label_color: AnsiValue,
    /// Color of numbers in the bottom bar's interval label
    pub bottom_bar_number_color: AnsiValue,
    /// Color of units in the bottom bar's interval label
    pub bottom_bar_unit_color: AnsiValue,
}

impl Theme {
    pub fn default() -> Self {
        Theme {
            top_bar_color: AnsiValue::grayscale(4),
            top_bar_number_color: AnsiValue::grayscale(18),
            top_bar_unit_color: AnsiValue::grayscale(12),
            tick_color: AnsiValue::grayscale(3),
            stream_even_background_color: AnsiValue::grayscale(0),
            stream_odd_background_color: AnsiValue::grayscale(1),
            stream_selected_background_color: AnsiValue::grayscale(2),
            stream_name_color: AnsiValue::grayscale(23),
            stream_name_selected_text_color: AnsiValue::grayscale(0),
            stream_name_selected_background_color: AnsiValue::grayscale(18),
            stream_description_color: AnsiValue::grayscale(16),
            stream_number_color: AnsiValue::grayscale(20),
            stream_unit_color: AnsiValue::grayscale(12),
            stream_graph_colors: vec![
                (AnsiValue::rgb(4, 0, 0), AnsiValue::rgb(5, 1, 1)),
                (AnsiValue::rgb(0, 4, 0), AnsiValue::rgb(1, 5, 1)),
                (AnsiValue::rgb(1, 1, 5), AnsiValue::rgb(2, 2, 5)),
                (AnsiValue::rgb(4, 4, 0), AnsiValue::rgb(5, 5, 1)),
                (AnsiValue::rgb(5, 0, 5), AnsiValue::rgb(5, 1, 5)),
                (AnsiValue::rgb(0, 4, 3), AnsiValue::rgb(1, 5, 4)),
            ],
            bottom_bar_color: AnsiValue::grayscale(15),
            bottom_bar_key_text_color: AnsiValue::grayscale(0),
            bottom_bar_key_background_color: AnsiValue::grayscale(20),
            bottom_bar_label_color: AnsiValue::grayscale(0),
            bottom_bar_number_color: AnsiValue::rgb(0, 0, 5),
            bottom_bar_unit_color: AnsiValue::rgb(0, 0, 2),
        }
    }
}
