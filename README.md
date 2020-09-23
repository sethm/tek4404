# About

This is a Tektronix 4404 emulator in the larval stage. There's nothing
to see here yet.

# Status

This is a work in progress and completely unusable. The project is
very young, in active development, and is skeletal.

# Usage

## Tests

The emulator is written in Rust, and requires Rust 1.31 or later to
compile. I have tried to avoid using Rust nightly features.

At the moment, the most interesting things are the unit tests, which
can be run with:

    $ cargo test

## Boot ROM

To execute the boot ROM using cargo, type:

    $ cargo run -- -b ./rom/boot.bin -s 20000 -l info

or, from a built binary:

    $ tek4404 -b ./rom/boot.bin -s 20000 -l info

This enters an infinite CPU execution loop that does 20,000 machine
cycles on each pass through the loop, with 100ms of idle time between
executions. To kill the emulator, just use ^C (Control-C).

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
