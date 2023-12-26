use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, RwLock};
use std::thread;



use rodio::{Decoder, OutputStream, Sink};
use crate::sound::audio_sink::AudioSink;

pub const RESOURCE_MUSIC_BACKGROUND : &str = "resources/alexander-nakarada-tavern-loop-one.mp3";

pub struct SoundSinks {
    bg_sink: AudioSink
}

pub fn build_sound_sinks() -> SoundSinks {
    let bg_sink = AudioSink::new();
    let mut sinks = SoundSinks { bg_sink };
    sinks.setup_background_music();
    return sinks;
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
            let file = BufReader::new(File::open(RESOURCE_MUSIC_BACKGROUND).unwrap());
            let looped_decoder = Decoder::new_looped(file).unwrap();

            let sink = sink_arc.write().unwrap();
            sink.append(looped_decoder);
            //sink.play();
        });
    }

    pub fn get_bg_sink(&self) -> &AudioSink {
        &self.bg_sink
    }

    pub fn get_bg_sink_mut(&mut self) -> &mut AudioSink {
        &mut self.bg_sink
    }
}

