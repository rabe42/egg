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
> egg 23:59:59  # Will start to play the sound just before midnight.

> egg 6m        # Will tell you that your Egg is Ok in 6 minutes

> EGG_SOUND="~/Music/Ac-DC/Hells-Bells.mp3" egg 3h # Will play your music in 3 hours
```

## Technically
It will block the command line, if you don't spawn it to the background.
There is no persistence or anything involved. The main implementation
effort goes to the command line argument parsing using regex. If you
escape blanks, this is considered as wrong.

## Caveats
If you terminate the process, e.g. by killing the terminal window, the
sound will be never played.
