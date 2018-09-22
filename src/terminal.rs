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

use std::io::{self, Write};
use std::thread;

use termion::event::Event;
use termion::input::{TermRead, MouseTerminal};
use termion::screen::AlternateScreen;
use termion::raw::IntoRawMode;
use termion::{self, cursor, clear};
use chan::{self, Receiver};
use chan_signal::{self, Signal};

// See https://vt100.net/docs/vt102-ug/chapter5.html#S5.5.2.8
const ENABLE_AUTO_WRAP: &str = "\x1B[?7h";
const DISABLE_AUTO_WRAP: &str = "\x1B[?7l";

pub struct Terminal {
    wrapper: Box<Write>,
    pub input: Receiver<Event>,
    pub resize: Receiver<Signal>,
}

impl Terminal {
    pub fn new() -> Self {
        // NOTE: If this line is below `thread::spawn`, resize events are not received
        // as documented in https://docs.rs/chan-signal/0.3.2/chan_signal/fn.notify.html
        let resize = chan_signal::notify(&[Signal::WINCH]);

        let (input_sender, input) = chan::async();

        thread::spawn(move || {
            for event in io::stdin().events() {
                input_sender.send(event.unwrap());
            }
        });

        let terminal = Terminal {
            wrapper: Box::new(MouseTerminal::from(AlternateScreen::from(io::stdout().into_raw_mode().unwrap()))),
            input,
            resize,
        };

        terminal.print(format!("{}{}{}{}", cursor::Hide, clear::All, cursor::Goto(1, 1), DISABLE_AUTO_WRAP));

        terminal
    }

    pub fn print(&self, output: impl AsRef<[u8]>) {
        io::stdout().write_all(output.as_ref()).unwrap();
        io::stdout().flush().unwrap();
    }

    pub fn size(&self) -> (usize, usize) {
        let (width, height) = termion::terminal_size().unwrap();
        (width as usize, height as usize)
    }
}

// NOTE: This could be simplified if https://github.com/redox-os/termion/pull/113 were merged
impl Drop for Terminal {
    fn drop(&mut self) {
        self.print(format!("{}{}", ENABLE_AUTO_WRAP, cursor::Show));
    }
}
