# Game Boy emulator powered by Dynamic Recompilation

## What is Dynamic Recompilation?

Dynamic Recompilation is a technique for running code designed for a different
processor architecture. It takes code compiled for one platform, and re-compiles
it for the host platform on demand. It is often used by high-performance
emulators and virtual machines like QEMU. it's also (probably) used by the
latest version of macOS to run x86 programs on Apple Silicon. That particular
use case is what inspired me to learn how Dynamic Recompilation works, and build
a simple emulator on those principles.

Many emulators work by interpreting code. They read each processor instruction,
and perform some simulation based on the intended behavior. This adds some
overhead, but often makes it easier to write emulators in high-level languages
and port to different systems. Dynamic Recompilation removes much of this
overhead, and can even provide opportunities for further optimization, but it
needs to be re-implemented for each host system. At the moment, this project
only targets x86 systems running Linux or Windows.

A Game Boy emulator doesn't need fancy techniques like Dynamic Recompilation to
run at full speed on modern computer systems. I've chosen to target the Game Boy
because it's a complex enough target to be fun and interesting, but simple
enough to be reasonably achievable. It's also a system I've built multiple
emulators for in the past, so I can focus on learning the DynaRec parts.

## How does Dynamic Recompilation work?

The core concept of Dynamic Recompilation is on-demand generation of native
machine code that then gets executed. When the emulator encounters a new block
of code, it reads the machine code up until a break point (a jump, function
call, or other flow-control change). It then translates that code from the
emulated platform to the host platform, and executes it. The result can be
cached, so that the next time that code block is encountered it doesn't need to
be re-translated.
