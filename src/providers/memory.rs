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

use systemstat::{Platform, System};

use stream::{Stream, StreamProvider};

const SWAP_TOTAL: &str = "SwapTotal";
const SWAP_FREE: &str = "SwapFree";

pub struct MemoryStreamProvider {}

impl StreamProvider for MemoryStreamProvider {
    fn streams(&self) -> Vec<Box<Stream>> {
        let mut streams = Vec::new();

        if let Ok(memory) = System::new().memory() {
            streams.push(Stream::new(
                "Mem",
                "Amount of physical memory (RAM) in use",
                move || {
                    if let Ok(memory) = System::new().memory() {
                        Some((memory.total - memory.free).as_usize() as f64)
                    } else {
                        None
                    }
                },
                Some(0.0),
                Some(memory.total.as_usize() as f64),
                "B",
                None,
                1,
                false,
            ));

            let meminfo = memory.platform_memory.meminfo;
            if meminfo.contains_key(SWAP_TOTAL) && meminfo.contains_key(SWAP_FREE) {
                streams.push(Stream::new(
                    "Swap",
                    "Amount of swap space in use",
                    move || {
                        if let Ok(memory) = System::new().memory() {
                            let meminfo = memory.platform_memory.meminfo;
                            Some((meminfo[SWAP_TOTAL] - meminfo[SWAP_FREE]).as_usize() as f64)
                        } else {
                            None
                        }
                    },
                    Some(0.0),
                    Some(meminfo[SWAP_TOTAL].as_usize() as f64),
                    "B",
                    None,
                    1,
                    false,
                ));
            }
        }

        streams
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_stream_provider() {
        let streams = MemoryStreamProvider {}.streams();
        assert!(!streams.is_empty());
    }
}
