// Hegemon - A modular system monitor
// Copyright (C) 2018-2019  Philipp Emanuel Weidmann <pew@worldwidemann.com>
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

#[macro_use]
extern crate crossbeam_channel;
extern crate regex;
extern crate sensors;
extern crate signal_hook;
extern crate systemstat;
extern crate termion;

mod model;
mod providers;
mod stream;
mod terminal;
mod theme;
mod view;

use crate::model::Application;
use crate::terminal::Terminal;
use crate::theme::Theme;

fn main() {
    let terminal = Terminal::new();
    let (width, height) = terminal.size();

    let mut application = Application::new(width, height, providers::streams());
    application.update_streams();

    let theme = Theme::default();
    terminal.print(application.render(&theme));

    let mut update = crossbeam_channel::tick(application.interval().duration);

    // Main event loop
    loop {
        select! {
            recv(terminal.input) -> event => {
                let interval_index = application.interval_index;

                if application.handle(&event.unwrap()) {
                    if !application.running {
                        break;
                    }
                    if application.interval_index != interval_index {
                        application.reset_streams();
                        application.update_streams();
                        update = crossbeam_channel::tick(application.interval().duration);
                    }
                    terminal.print(application.render(&theme));
                } else {
                    // Bell
                    terminal.print("\x07");
                }
            },
            recv(terminal.resize) -> _ => {
                let (width, height) = terminal.size();
                application.resize(width, height);
                terminal.print(application.render(&theme));
            },
            recv(terminal.terminate) -> _ => {
                break;
            },
            recv(update) -> _ => {
                application.update_streams();
                terminal.print(application.render(&theme));
            },
        }
    }
}
