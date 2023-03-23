use soloud::*;
use std::error::Error;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

fn process_command_line() -> Result<Duration, Box<dyn Error>>
{
    Ok(Duration::from_secs(0))
}

fn play_sound() -> Result<(), Box<dyn Error>> {
    let sl = Soloud::default()?;
    let mut sound = Wav::default();
    sound.load(Path::new("sounds/Hells-Bells.mp3"))?;
    sl.play(&sound);
    while sl.voice_count() > 0 {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    Ok(())
}

fn usage(error: Box<dyn Error>) {
    eprintln!("We have had a problem: '{}'", error)
}

/// Plays the provided sound!
fn main() -> Result<(), Box<dyn Error>>
{
    match process_command_line() {
        Ok(time_to_wait) => {
            sleep(time_to_wait);
            play_sound()
        },
        Err(err) => {
            usage(err);
            Ok(())
        }
    }
}
