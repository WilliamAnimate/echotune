# The echotune music player

A tui music player that gives some thought on time spent on the CPU.

> [!IMPORTANT]
> echotune is not a production-ready music player! However, feel free to play around with it if you figure out the ropes.

## Does that mean it's performant and wont lag my system?

Yes! The bulk (as of right now, all of) echotune's code is written on [this](https://www.ordinateursarabais.com/produit/acer-es1-521-40hc-hdmi-6-go-ram-1-tb/)[^1]. it takes about 200µs to draw to the tty and i've never seen it use >7% CPU usage.

## Safe with Rust

Rust (alongside Zig) are the future of programming languages whether you like it or not. No longer will you have to choose between performance (C) or safe code (every other high level language that exists).

Because echotune is written in Rust, you need not worry about getting a remote code execution from a specifically crafted .flac file.

## Vi-inspired

Because echotune runs in the terminal, echotune comes with vi-like keybindings. That means if you are the based ones using a modal editor based on vi or vim then you will find echotune an easy adaptation.

[^1]: IOW, your modern Intel core 15 gen CPU @ 42 GHz with DDR7 RAM with a 32 TB NVMe SSD and liquid-cooled machine running the most bloated (GNU/)Linux distro (or Windows...) well surpasses the system requirements for running echotune, and that you will notice no difference in performance, if it can run pretty well on this AMD Quad-Core A4 processor @ 1.8 GHz

