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

use std::collections::HashMap;
use std::time::Duration;

use termion::event::{Event, Key, MouseButton, MouseEvent};

use stream::Stream;

pub struct Application {
    pub running: bool,
    pub width: usize,
    pub height: usize,
    pub screen: Screen,
    pub streams: Vec<StreamWrapper>,
    pub selection_index: usize,
    pub scroll_index: usize,
    pub scroll_anchor: ScrollAnchor,
    intervals: Vec<Interval>,
    pub interval_index: usize,
    // The two parts of the map value contain
    // the left/right-aligned menu items, respectively
    menus: HashMap<Screen, (Vec<MenuItem>, Vec<MenuItem>)>,
}

impl Application {
    pub fn new(width: usize, height: usize, streams: Vec<Box<Stream>>) -> Self {
        let mut menus = HashMap::new();

        menus.insert(
            Screen::Main,
            (
                vec![
                    MenuItem::new("\u{1F805}\u{1F807}", "Select"),
                    MenuItem::new("Space", "Expand"),
                    MenuItem::new("S", "Streams"),
                    MenuItem::new("+-", "Interval"),
                ],
                vec![MenuItem::new("Q", "Quit")],
            ),
        );

        menus.insert(
            Screen::Streams,
            (
                vec![
                    MenuItem::new("\u{1F805}\u{1F807}", "Select"),
                    MenuItem::new("Space", "Toggle"),
                    MenuItem::new("+-", "Reorder"),
                ],
                vec![MenuItem::new("Esc", "Done")],
            ),
        );

        Application {
            running: true,
            width,
            height,
            screen: Screen::Main,
            streams: streams.into_iter().map(StreamWrapper::new).collect(),
            selection_index: 0,
            scroll_index: 0,
            scroll_anchor: ScrollAnchor::Top,
            intervals: vec![
                Interval::new(1, 10),
                Interval::new(2, 10),
                Interval::new(3, 10),
                Interval::new(5, 12),
                Interval::new(10, 12),
                Interval::new(30, 10),
                Interval::new(60, 10),
                Interval::new(300, 12),
            ],
            interval_index: 2,
            menus,
        }
    }

    pub fn interval(&self) -> Interval {
        self.intervals[self.interval_index]
    }

    pub fn menu(&self) -> (Vec<MenuItem>, Vec<MenuItem>) {
        self.menus[&self.screen].clone()
    }

    pub fn active_streams(&self) -> Vec<&StreamWrapper> {
        self.streams.iter().filter(|s| s.active).collect()
    }

    pub fn handle(&mut self, event: &Event) -> bool {
        match self.screen {
            Screen::Main => match event {
                Event::Key(key) => match key {
                    Key::Up => {
                        if self.selection_index > 0 {
                            self.selection_index -= 1;
                            let selection_index = self.selection_index;
                            self.scroll_to_stream(selection_index);
                            return true;
                        }
                    }
                    Key::Down => {
                        if self.selection_index < self.active_streams().len() - 1 {
                            self.selection_index += 1;
                            let selection_index = self.selection_index;
                            self.scroll_to_stream(selection_index);
                            return true;
                        }
                    }
                    Key::Char(' ') => {
                        {
                            let stream = self
                                .streams
                                .iter_mut()
                                .filter(|s| s.active)
                                .nth(self.selection_index)
                                .unwrap();
                            stream.expanded = !stream.expanded;
                        }
                        let selection_index = self.selection_index;
                        self.scroll_to_stream(selection_index);
                        return true;
                    }
                    Key::Char('s') => {
                        self.screen = Screen::Streams;
                        return true;
                    }
                    Key::Char('+') => {
                        if self.interval_index < self.intervals.len() - 1 {
                            self.interval_index += 1;
                            return true;
                        }
                    }
                    Key::Char('-') => {
                        if self.interval_index > 0 {
                            self.interval_index -= 1;
                            return true;
                        }
                    }
                    Key::Char('q') => {
                        self.running = false;
                        return true;
                    }
                    _ => {}
                },
                Event::Mouse(MouseEvent::Press(mouse_button, _, _)) => match mouse_button {
                    MouseButton::WheelUp => {
                        return self.handle(&Event::Key(Key::Down));
                    }
                    MouseButton::WheelDown => {
                        return self.handle(&Event::Key(Key::Up));
                    }
                    _ => {}
                },
                _ => {}
            },

            Screen::Streams => match event {
                Event::Key(key) => match key {
                    Key::Up => {}
                    Key::Down => {}
                    Key::Char(' ') => {}
                    Key::Char('+') => {}
                    Key::Char('-') => {}
                    Key::Esc => {
                        self.screen = Screen::Main;
                        return true;
                    }
                    _ => {}
                },
                Event::Mouse(MouseEvent::Press(mouse_button, _, _)) => match mouse_button {
                    MouseButton::WheelUp => {}
                    MouseButton::WheelDown => {}
                    _ => {}
                },
                _ => {}
            },
        }

        false
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }

    fn scroll_to_stream(&mut self, index: usize) {
        let mut stream_count = 0;
        let mut available_height = self.height - 2;

        {
            let active_streams = self.active_streams();

            let streams = match self.scroll_anchor {
                ScrollAnchor::Top => active_streams[self.scroll_index..].iter().collect::<Vec<_>>(),
                ScrollAnchor::Bottom => active_streams[..=self.scroll_index].iter().rev().collect::<Vec<_>>(),
            };

            for stream in streams {
                let height = stream.height();
                if height > available_height {
                    break;
                }
                stream_count += 1;
                available_height -= height;
            }
        }

        // Only count streams beyond the first
        if stream_count > 0 {
            stream_count -= 1;
        }

        // Indices of the first and last streams that are *completely* visible
        let (top_index, bottom_index) = match self.scroll_anchor {
            ScrollAnchor::Top => (self.scroll_index, self.scroll_index + stream_count),
            ScrollAnchor::Bottom => (self.scroll_index - stream_count, self.scroll_index),
        };

        if index < top_index {
            self.scroll_index = index;
            self.scroll_anchor = ScrollAnchor::Top;
        } else if index > bottom_index {
            self.scroll_index = index;
            self.scroll_anchor = ScrollAnchor::Bottom;
        }
    }

    pub fn update_streams(&mut self) {
        for stream in &mut self.streams {
            if stream.active {
                let value = stream.stream.value();

                if let Some(number) = value {
                    assert!(number.is_finite());
                    if let Some(min) = stream.stream.min() {
                        assert!(number >= min);
                    }
                    if let Some(max) = stream.stream.max() {
                        assert!(number <= max);
                    }
                }

                stream.values.push(value);
            }
        }
    }

    pub fn reset_streams(&mut self) {
        for stream in &mut self.streams {
            // TODO: Reset stream's internal state
            stream.values.clear();
        }
    }
}

#[derive(PartialEq, Eq, Hash)]
pub enum Screen {
    Main,
    Streams,
}

pub struct StreamWrapper {
    pub stream: Box<Stream>,
    pub values: Vec<Option<f64>>,
    pub active: bool,
    pub expanded: bool,
}

impl StreamWrapper {
    fn new(stream: Box<Stream>) -> Self {
        StreamWrapper {
            stream,
            values: vec![],
            active: true,
            expanded: false,
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum ScrollAnchor {
    Top,
    Bottom,
}

#[derive(Copy, Clone)]
pub struct Interval {
    pub duration: Duration,
    pub tick_spacing: usize,
}

impl Interval {
    fn new(seconds: u64, tick_spacing: usize) -> Self {
        Interval {
            duration: Duration::from_secs(seconds),
            tick_spacing,
        }
    }
}

#[derive(Clone)]
pub struct MenuItem {
    pub keys: String,
    pub label: String,
}

impl MenuItem {
    fn new(keys: impl Into<String>, label: impl Into<String>) -> Self {
        MenuItem {
            keys: keys.into(),
            label: label.into(),
        }
    }
}
