use soloud::*;
use std::error::Error;
use std::path::Path;

/// Plays the provided sound!
fn main() -> Result<(), Box<dyn Error>>
{
    let sl = Soloud::default()?;
    let mut sound = Wav::default();
    sound.load(Path::new("sounds/Hells-Bells.mp3"))?;
    sl.play(&sound);
    while sl.voice_count() > 0 {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    Ok(())
}
