# About

The Tektronix 4404, code-named "Pegasus", was a mid-1980s AI
workstation. Based around the Motorola 68010 CPU, it ran Smalltalk-80,
supported up to 2 MB of RAM, and had a 45MB hard drive.

# Status

This is a work in progress and completely unusable. The project is
very young, in active development, and is skeletal.

As you can see, it does not yet work:

![Tektronix 4404 Main Window](/doc/screenshots/screenshot.png?raw=true)

# Requirements

The 4404 emulator uses SDL2, so you'll need SDL2 development libraries
to build it.

# Usage

## Tests

The emulator is written in Rust, and requires Rust 1.31 or later to
compile. I have tried to avoid using Rust nightly features.

At the moment, the most interesting things are the unit tests, which
can be run with:

    $ cargo test

## Boot ROM

To execute the boot ROM using cargo, type:

    $ cargo run -- -b ./rom/boot.bin -s 20000 -i 25 -l info

or, from a built binary:

    $ tek4404 -b ./rom/boot.bin -s 20000 -i 25 -l info

This enters an infinite CPU execution loop that does 20,000 machine
cycles on each pass through the loop, with 25ms of idle time between
executions. To kill the emulator, just use ^C (Control-C) or
close the main display window.

At the moment, the only peripheral that is emulated is the debug ACIA.
You can connect to the debug ACIA and issue interactive commands by
telnetting to localhost, port 9090. You can change the default listening
address and port with the --address and --port options.

# Credits

The Tektronix 4404 emulator uses [the Musashi Motorola 68000
core](https://github.com/kstenerud/Musashi).  Musashi is Copyright
1998-2002 by Karl Stenerud, and is distributed under the MIT License.

# License

Copyright 2020 Seth Morabito &lt;web@loomcom.com&gt;

Permission is hereby granted, free of charge, to any person obtaining
a copy of this software and associated documentation files (the
"Software"), to deal in the Software without restriction, including
without limitation the rights to use, copy, modify, merge, publish,
distribute, sublicense, and/or sell copies of the Software, and to
permit persons to whom the Software is furnished to do so, subject to
the following conditions:

The above copyright notice and this permission notice shall be
included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
