use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use log::log;
use rodio::{Decoder, OutputStream, source::Source, OutputStreamHandle, Sink, PlayError};

pub struct SoundSinks {
    os : Option<Arc<OutputStream>>,
    pub bg_sink: Option<Arc<RwLock<Sink>>>,
}

pub fn build_sound_sinks() -> SoundSinks {
    let mut sinks = SoundSinks { os:None, bg_sink: None }; // bg_sink: None, bg_thread: None };
    sinks.setup_background_music();
    return sinks;
}

impl SoundSinks {

    pub fn setup_background_music(&mut self) {
        log::info!("Starting background music..");
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        self.os = Some(Arc::new(stream));
        let sink_arc = Arc::new(RwLock::new(Sink::try_new(&stream_handle).unwrap()));
        self.bg_sink = Some(sink_arc.clone());

        thread::spawn(move || {
            let stream_handle = stream_handle;
            let file = BufReader::new(File::open("resources/alexander-nakarada-tavern-loop-one.mp3").unwrap());
            let looped_decoder = Decoder::new_looped(file).unwrap();

            let sink = sink_arc.write().unwrap();
            sink.append(looped_decoder);
            sink.play();
        });
    }

    pub fn play_background(&mut self) {
        if let Some(sink) = &self.bg_sink {
            let writeable = sink.write();
            if let Ok(w) = writeable{
                log::info!("Handling background music...");
                // TODO pass a sound config into here
                w.set_volume(0.5);
            } else {
                log::info!("No write lock.");
            }

        } else {
            log::info!("No bg sink to manage.");
        }
    }
}

