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

#[macro_use]
extern crate chan;
extern crate chan_signal;
extern crate regex;
extern crate sensors;
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

    let input = &terminal.input;
    let resize = &terminal.resize;
    let mut update = chan::tick(application.interval().duration);

    // Main event loop
    loop {
        let mut interval_changed = false;

        chan_select! {
            input.recv() -> event => {
                let interval_index = application.interval_index;

                if application.handle(&event.unwrap()) {
                    if !application.running {
                        break;
                    }
                    if application.interval_index != interval_index {
                        application.reset_streams();
                        application.update_streams();
                        interval_changed = true;
                    }
                    terminal.print(application.render(&theme));
                } else {
                    // Bell
                    terminal.print("\x07");
                }
            },
            resize.recv() => {
                let (width, height) = terminal.size();
                application.resize(width, height);
                terminal.print(application.render(&theme));
            },
            update.recv() => {
                application.update_streams();
                terminal.print(application.render(&theme));
            },
        }

        if interval_changed {
            // Ideally, this would happen inside the `chan_select` invocation above,
            // but the macro shadows the binding of `update` so the outer declaration
            // cannot be accessed
            update = chan::tick(application.interval().duration);
        }
    }
}
