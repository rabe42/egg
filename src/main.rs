use std::error::Error;
use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};
use std::env;
use regex::{Regex, Captures};
use thiserror::Error;
use chrono::Local;
use soloud::{AudioExt, LoadExt, Wav, Soloud};

use std::io::{Write, stdout};
use crossterm::{QueueableCommand, cursor, terminal, ExecutableCommand};

#[cfg(target_family="unix")]
use nix::unistd::{tcgetpgrp, getpgrp};

/// Compile-time (unit-test) validated regex for command line interface.
const HHMMSS_REGEX: &str = r"^(?P<hours>\d{2}):(?P<minutes>\d{2})(:(?P<seconds>\d{2}))?$";
const H_M_S_REGEX: &str = r"^((?P<hours>\d+)h)?((?P<minutes>\d+)m)?((?P<seconds>\d+)s)?$";
const H_M_REGEX: &str = r"^((?P<hours>\d+)h)?((?P<minutes>\d+)[m]*)?$";
const M_S_REGEX: &str = r"^((?P<minutes>\d+)m)?((?P<seconds>\d+)[s]*)?$";

/// The static bound sound file, included for save distribution.
const SOUND: &[u8] = include_bytes!("../sounds/mixkit-service-bell-double-ding-588.wav");

/// A simple error for managing issues in the parameters.
#[derive(Debug, Error)]
enum EggError {
    #[error("Invalid parameters: '{0}'")]
    InvalidParameters(String),
    #[error("Time already passed: {0}")]
    TimeAlreadyPassed(String),
    #[error("Time format not valid: {0}")]
    InvalidTimeFormat(String),
}

/// Detects, if the process runs at the point of time of it's calling, in the foreground.
fn is_foreground_process() -> bool
{
    #[cfg(target_family="unix")]
    if let Ok(pid) = tcgetpgrp(std::io::stdin()) {
        if pid.as_raw() == -1 {
            false
        } else {
            pid == getpgrp()
        }
    } else {
        false
    }

    #[cfg(target_family="windows")]
    true
}

/// Creates a single string from the command line arguments.
fn consolidate_command_line(args: Vec<String>) -> String
{
    let mut time_defintion = String::from("");
    for arg in args {
        time_defintion.push_str(&arg);
    }
    time_defintion
}

/// Given the regex, only number representations will be provided.
fn get_number(representation: &Option<regex::Match>) -> Result<u32, Box<dyn Error>>
{
    if let Some(rmatch) = representation {
        match rmatch.as_str().parse() {
            Ok(i) => Ok(i),
            Err(err) => Err(Box::new(err)),
        }
    }
    else {
        Ok(0)
    }
}

/// Calculates the duration from the absolute values in the captures.
fn duration_from_absolute(captures: Captures) -> Result<Duration, Box<dyn Error>>
{
    let hours = get_number(&captures.name("hours"))?;
    let minutes = get_number(&captures.name("minutes"))?;
    let seconds = get_number(&captures.name("seconds"))?;
    let now = Local::now().naive_local();
    let alert_time = match Local::now().date_naive().and_hms_opt(hours, minutes, seconds) {
        Some(time) => time,
        None => return Err(Box::new(EggError::InvalidTimeFormat(format!("{hours:0>2}:{minutes:0>2}:{seconds:0>2}"))))
    };
    let duration = alert_time.signed_duration_since(now);
    if duration.num_seconds() > 0 {
        Ok(alert_time.signed_duration_since(now).to_std().unwrap())
    }
    else {
        Err(Box::new(EggError::TimeAlreadyPassed(format!("{hours:0>2}:{minutes:0>2}:{seconds:0>2}"))))
    }
}

/// Calculates the duration to wait before playing the sound. The captures must be provided from a
/// regex capture() call.
fn duration_from_relative(captures: Captures) -> Result<Duration, Box<dyn Error>>
{
    let hours = get_number(&captures.name("hours"))?;
    let minutes = get_number(&captures.name("minutes"))?;
    let seconds = get_number(&captures.name("seconds"))?;
    // As the hours, minutes and seconds are all unsigned (u32) the result will be inside of u64!
    Ok(Duration::from_secs((hours*3600 + minutes*60 + seconds).into()))
}

fn duration_from_hm_relative(captures: Captures) -> Result<Duration, Box<dyn Error>>
{
    let hours = get_number(&captures.name("hours"))?;
    let minutes = get_number(&captures.name("minutes"))?;
    Ok(Duration::from_secs((hours*3600 + minutes*60).into()))
}

fn duration_from_ms_relative(captures: Captures) -> Result<Duration, Box<dyn Error>>
{
    let minutes = get_number(&captures.name("minutes"))?;
    let seconds = get_number(&captures.name("seconds"))?;
    // As the hours, minutes and seconds are all unsigned (u32) the result will be inside of u64!
    Ok(Duration::from_secs((minutes*60 + seconds).into()))
}

/// Processes the command line and returns an duration.
fn process_command_line() -> Result<Duration, Box<dyn Error>>
{
    let mut args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        Ok(Duration::from_secs(0))
    }
    else {
        // We doen need the first argument (command name)
        args.remove(0);

        // The complete command line should be considered as one argument.
        let time_defintion = consolidate_command_line(args);

        // Parse the string with constant regex for matching the command line arguments.
        let absolute = Regex::new(HHMMSS_REGEX).unwrap();
        let relative = Regex::new(H_M_S_REGEX).unwrap();
        let hm_short = Regex::new(H_M_REGEX).unwrap();
        let ms_short = Regex::new(M_S_REGEX).unwrap();
        if let Some(captures) = absolute.captures(&time_defintion) {
            duration_from_absolute(captures)
        }
        else if let Some(captures) = relative.captures(&time_defintion) {
            duration_from_relative(captures)
        }
        else if let Some(captures) = hm_short.captures(&time_defintion) {
            duration_from_hm_relative(captures)
        }
        else if let Some(captures) = ms_short.captures(&time_defintion) {
            duration_from_ms_relative(captures)
        }
        else {
            Err(Box::new(EggError::InvalidParameters(time_defintion)))
        }
    }
}

/// Play the sound, provided by the user.
fn play_sound(sound_file: &str) -> Result<(), Box<dyn Error>> {

    let sl = Soloud::default()?;
    let mut sound = Wav::default();
    sound.load(Path::new(sound_file))?;
    sl.play(&sound);
    while sl.voice_count() > 0 {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    Ok(())
}

/// Play the sound, stored in the binary.
fn play_internal_sound() -> Result<(), Box<dyn Error>> {

    let sl = Soloud::default()?;
    let mut sound = Wav::default();
    sound.load_mem(SOUND)?;
    sl.play(&sound);
    while sl.voice_count() > 0 {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    Ok(())
}

/// Creates a string with the duration in HH:MM:SS format.
fn format_duration(duration: Duration) -> String {
    let s = duration.as_secs() % 60;
    let m = duration.as_secs() / 60 % 60;
    let h = duration.as_secs() / 60 / 60;
    format!("{:0>2}:{:0>2}:{:0>2}", h, m, s)
}

/// Put a countdown of the remaining waiting time to the command line.
/// It is constructed as a drop-in for the sleep function.
fn countdown(duration: Duration) {
    let start = Instant::now();
    let mut stdout = stdout();
    let mut passed = start.elapsed();

    stdout.execute(cursor::Hide).unwrap();
    while passed < duration {
        let delta = duration - passed;

        stdout.queue(cursor::SavePosition).unwrap();
        stdout.write_all(format_duration(delta).as_bytes()).unwrap();
        stdout.queue(cursor::RestorePosition).unwrap();
        stdout.flush().unwrap();

        sleep(Duration::from_millis(1000));
        passed = start.elapsed();

        stdout.queue(cursor::RestorePosition).unwrap();
        stdout.queue(terminal::Clear(terminal::ClearType::FromCursorDown)).unwrap();
    }
    stdout.execute(cursor::Show).unwrap();
}

/// Print an usage message to the stderr.
fn usage(error: Box<dyn Error>) -> Result<(), Box<dyn Error>> {
    eprintln!("We have had a problem: '{}'", error);
    eprintln!("
Usage:
    egg HH:MM:SS or HHhMMmSSs

Example:
    egg 19:00
    egg 5m30s

Environment Variables
    EGG_SOUND <path to a sound file>
");
    Err(error)
}

/// Plays the provided sound!
fn main() -> Result<(), Box<dyn Error>>
{
    match process_command_line() {
        Ok(time_to_wait) => {
            if is_foreground_process() {
                countdown(time_to_wait);
            } else {
                sleep(time_to_wait);
            }
            match env::var("EGG_SOUND") {
                Ok(f) => play_sound(f.as_str()),
                Err(_) => play_internal_sound(),
            }
        },
        Err(err) => {
            usage(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_absolute_regex() {
        let abs = Regex::new(HHMMSS_REGEX).unwrap();
        let caps = abs.captures("19:30:00").unwrap();
        assert_eq!("19", &caps["hours"]);
        assert_eq!("30", &caps["minutes"]);
        assert_eq!("00", &caps["seconds"]);
        let caps = abs.captures("22:59").unwrap();
        assert_eq!("22", &caps["hours"]);
        assert_eq!("59", &caps["minutes"]);
        assert!(&caps.name("seconds").is_none());
    }

    #[test]
    fn test_relative_regex() {
        let rel = Regex::new(H_M_S_REGEX).unwrap();
        let caps = rel.captures("5m49s").unwrap();
        assert!(&caps.name("hours").is_none());
        assert_eq!("5", &caps["minutes"]);
        assert_eq!("49", &caps["seconds"]);
        let caps = rel.captures("1h").unwrap();
        assert_eq!("1", &caps["hours"]);
        assert!(&caps.name("minutes").is_none());
        assert!(&caps.name("seconds").is_none());
        let caps = rel.captures("6m").unwrap();
        assert!(&caps.name("hours").is_none());
        assert_eq!("6", &caps["minutes"]);
        assert!(&caps.name("seconds").is_none());
    }

    #[test]
    fn test_hm_regex() {
        let r = Regex::new(H_M_REGEX).unwrap();
        let caps = r.captures("10h5").unwrap();
        assert_eq!("10", &caps["hours"]);
        assert_eq!("5", &caps["minutes"]);
    }

    #[test]
    fn test_ms_regex() {
        let r = Regex::new(M_S_REGEX).unwrap();
        let caps = r.captures("19m3").unwrap();
        assert_eq!("19", &caps["minutes"]);
        assert_eq!("3", &caps["seconds"]);
    }

    #[test]
    fn test_absolute_error_handling() {
        let re = Regex::new(HHMMSS_REGEX).unwrap();
        let caps = re.captures("19:99:23").unwrap();
        duration_from_absolute(caps).unwrap_err();
    }
}
