use std::fs::{File, ReadDir};
use std::io::{BufReader, Error, ErrorKind};
use std::sync::{Arc, RwLock};
use std::{fs, thread};
use log::{error, info};
use rand::{Rng, SeedableRng};
use rand::seq::IteratorRandom;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;

use rodio::{Decoder, OutputStream, Sink};
use termion::event::Key::F;
use tokio::io::AsyncWriteExt;
use tokio::runtime::Runtime;

use crate::sound::audio_sink::AudioSink;

pub const RESOURCE_MUSIC_BACKGROUND_FOLDER: &str = "resources/background";

pub struct SoundSinks {
    bg_sink: AudioSink
}

pub fn build_sound_sinks() -> SoundSinks {
    let bg_sink = AudioSink::new();
    let mut sinks = SoundSinks { bg_sink };
    sinks.setup_background_music();
    return sinks;
}

pub fn pick_background_track(paths: &mut ReadDir, rng: &mut impl Rng) -> Result<std::io::BufReader<std::fs::File>, std::io::Error> {
    let chosen_dir = paths.choose(rng);
    if let Some(dir_entry) = chosen_dir {
        match dir_entry {
            Ok(de) => {
                let file_path = de.path();
                info!("Picked a new background track: {:?}", file_path);
                let path = de.path();
                return Ok(BufReader::new(File::open(path).unwrap()))
            },
            Err(e) => {
                return Err(e)
            }
        }
    } else {
        Err(Error::new(ErrorKind::Other, "Unable to choose a file path (ran out of directory paths?)"))
    }
}

pub fn play_background_music() {

}


impl SoundSinks {
    pub fn setup_background_music(&mut self) {
        log::info!("Starting background music..");
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        self.bg_sink.set_os(Some(Arc::new(stream)));
        let sink_arc = Arc::new(RwLock::new(Sink::try_new(&stream_handle).unwrap()));
        self.bg_sink.set_sink(Some(sink_arc.clone()));

        thread::spawn(move || {
            let _stream_handle = stream_handle;
            let sink = sink_arc.write().unwrap();
            loop {
                if sink.empty() {
                    let mut paths = fs::read_dir(RESOURCE_MUSIC_BACKGROUND_FOLDER).unwrap();
                    let mut rng = Pcg64::from_entropy();
                    match pick_background_track(&mut paths, &mut rng) {
                        Ok(track) => {
                            let decoder = Decoder::new_mp3(track).unwrap();
                            sink.append(decoder);
                        },
                        Err(e) => {
                            error!("Failed to pick background track: {}", e)
                        }
                    }
                }
            }
        });
    }

    pub fn get_bg_sink(&self) -> &AudioSink {
        &self.bg_sink
    }

    pub fn get_bg_sink_mut(&mut self) -> &mut AudioSink {
        &mut self.bg_sink
    }
}

