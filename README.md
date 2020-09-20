# About

This is a Tektronix 4404 emulator in the larval stage. There's nothing
to see here yet.

# Status

This is a work in progress and completely unusable. The project is
very young, in active development, and is skeletal.

# Usage

The emulator is written in Rust, and requires Rust 1.31 or later to
compile. I have tried to avoid using Rust nightly features.

At the moment, the most interesting things are the unit tests, which
can be run with:

    $ cargo test
    
To execute the boot rom using cargo, type:

    $ cargo run -- -b ./rom/boot.bin -s 200 -l debug

or, from a built binary:

    $ tek4404 -b ./rom/boot.bin -s 200 -l debug
    
This will execute 200 machine cycles at the DEBUG log level. Example output:

```
$ tek4404 -b ./rom/boot.bin -s 200 -l debug
[INFO ][tek4404] INITIALIZING
[INFO ][    bus] Loaded 32768 bytes from ./rom/boot.bin
[INFO ][tek4404] BOOTING
[DEBUG][    cpu] 00741b22:    move.b  #$1, $740000.l
[DEBUG][    cpu] 00741b2a:    move.l  #$740000, D0
[DEBUG][    cpu] 00741b30:    movea.l D0, A6
[DEBUG][    cpu] 00741b32:    movec D0, VBR; (1+)
[DEBUG][    cpu] 00741b36:    move.b  #$e, (A6)
[DEBUG][    cpu] 00741b3a:    clr.l   D7
[DEBUG][    cpu] 00741b3c:    movea.l A6, A3
[DEBUG][    cpu] 00741b3e:    adda.l  #$ffec, A3
[DEBUG][    cpu] 00741b44:    move.w  ($a,A3), D0
[DEBUG][    cpu] 00741b48:    not.w   D0
[DEBUG][    cpu] 00741b4a:    move.w  ($8,A3), D1
[DEBUG][    cpu] 00741b4e:    eor.w   D1, D0
[DEBUG][    cpu] 00741b50:    beq     $741b6a
[DEBUG][    cpu] 00741b6a:    clr.l   D1
[DEBUG][    cpu] 00741b6c:    subq.b  #1, D1
[DEBUG][    cpu] 00741b6e:    move.l  D1, D2
[DEBUG][    cpu] 00741b70:    move.l  D1, D3
[DEBUG][    cpu] 00741b72:    move.l  D1, D4
[DEBUG][    cpu] 00741b74:    clr.l   D5
[DEBUG][    cpu] 00741b76:    move.w  ($6,A3), D5
[INFO ][tek4404] 164 CYCLES COMPLETED IN 29.2044ms (5.6552 cycles/ms)
```

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
