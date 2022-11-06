use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use log::log;
use rodio::{Decoder, OutputStream, source::Source, OutputStreamHandle, Sink, PlayError};

pub struct SoundSinks {
    //pub bg_source: Option<Source<Decoder<R>>>,
    //pub bg_sink: Option<Arc<Mutex<Sink>>>,
    //pub bg_thread: Option<thread::JoinHandle<()>>
}

pub fn build_sound_sinks() -> SoundSinks {
    let mut sinks = SoundSinks {}; // bg_sink: None, bg_thread: None };
    sinks.setup_background_music();
    return sinks;
}

impl SoundSinks {

    pub fn setup_background_music(&mut self) {
        log::info!("Starting background music..");

        let handle = thread::spawn(move || {
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();

            let file = BufReader::new(File::open("resources/alexander-nakarada-tavern-loop-one.mp3").unwrap());
            let source = Decoder::new(file).unwrap();
            sink.append(source);

            sink.play();

            loop {
                log::debug!("Pausing background music thread..");
                // Give 900ms back to the main thread
                thread::sleep(Duration::from_millis(900));
            }
        });

        //let arc = Arc::new(Mutex::new(sink));
        //self.bg_sink = Some(arc);
    }

    pub fn play_background(&mut self) {
        // TODO test accessing Sink volume etc/thread pausing
        //let inner_arc = Arc::clone(&self.bg_sink.as_ref().unwrap());
        //let sink = inner_arc.lock().unwrap();

        // Give 100ms to the background music thread
        thread::sleep(Duration::from_millis(100));
    }
}

