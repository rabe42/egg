# EGG

## Purpose

The purpose is to provide a small command line tool, which starts a timer
and plays a sound, if the time is up.

You may provide absolute points in time in the following formats:
- `hh:mm:ss` (Will play the sound at the absolute point in time in the
  timezone of the caller.)
- `hh:mm`

Or you provide a delta like so:
* `120m` (Will play the sound in 12 minutes)
* `1h 12m 3s` (Will play the sound in 1 hour 12 minutes and 3 seconds.)
* `1h12m3s`

## Technically
It is the most simplistic implementation possible. It will block the
command line, if you don't spawn it to the background. There is no
persistence or anything involved. The main implementation effort goes to
the command line argument parsing.

## Caveats
If you terminate the process, e.g. by killing the terminal window, the
sound will be never played.
