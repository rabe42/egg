# EGG

## Purpose
The purpose is to provide a small command line tool, which starts a timer
and plays a sound, if the time is up. It follows the KISS principles, by
focussing on this one purpose.

You may provide absolute points in time in the following formats:
- `hh:mm:ss` (Will play the sound at the absolute point in time in the
  timezone of the caller.)
- `hh:mm`

Or you provide a delta like so:
* `12m` (Will play the sound in 12 minutes)
* `1h 12m 3s` (Will play the sound in 1 hour 12 minutes and 3 seconds.)
* `1h12m3s`

If you don't like the provided sound, you may specify your own sound using
the environment variable `EGG_SOUND`.

## Examples

```sh
egg 23:59:59  # Will start to play the sound just before midnight.

egg 6m        # Will tell you that your Egg is Ok in 6 minutes

EGG_SOUND="~/Music/Ac-DC/Hells-Bells.mp3" egg 3h # Will play your music in 3 hours
```

## Technically
It will block the command line, if you don't spawn it to the background.
There is no persistence or anything involved. The main implementation
effort goes to the command line argument parsing using regex. If you
escape blanks, this is considered as wrong.

## Media
The included sound is a free sound file from [Mixkit](https://mixkit.co/free-sound-effects/bell/).

## Caveats
If you terminate the process, e.g. by killing the terminal window, the
sound will be never played.

The program detects, if it was started running on the terminal in the
foreground or in the background. If it is running in the foreground, a
countdown will be shown. Be aware, that there is a difference between
running in background and suspending a foreground process. If the
foreground process is suspended, no CPU is committed to it until it is
brought back to the foreground. In the suspended state, no sound will be
played until the process is brought back to the foreground.

## Compiling
Please be aware, that the `soloud` library requires a `cmake`.

The release build tries to optimize for size as far as possible with the
standard flags (see `Cargo.toml#profile.release`). All other steps will only
gain another 10-15% but add severe complexity to the build process.

To make these additional optimization you may optimize the used `libc` for the
needs of this application. To do so perform the following steps for your
build:

```sh
rustc -vV | grep host
```

This will provide you with the information regarding your current
environment e.g.: `host: x86_64-unknown-linux-gnu` for a Linux machine or
`x86_64-apple-darwin` for a MacOS system with x86 architecture.

Than you may compile the libc with the following command in this case
optimized for a Linux system:

```sh
cargo +nightly build -Z build-std=std,panic_abort --target x86_64-unknown-linux-gnu --release
```

For this to work, you must install a nightly tool chain, as only this
nightly cargo version supports the `-Z` option. In addition you need also
the sources to compile the libc.

```sh
rustup toolchain install nightly
rustup component add rust-src --toolchain nightly
```

Be aware that the result of this optimization is located in the folder:
`target/x86_64-unknown-linux-gnu/release/`.

