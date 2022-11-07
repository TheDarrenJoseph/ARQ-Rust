use std::sync::{Arc, RwLock};
use rodio::{OutputStream, Sink};

pub struct AudioSink {
    os : Option<Arc<OutputStream>>,
    sink: Option<Arc<RwLock<Sink>>>
}

impl AudioSink {
    pub fn new() -> AudioSink {
        AudioSink { os:None, sink: None }
    }

    pub fn get_os(&self) -> &Option<Arc<OutputStream>> {
        &self.os
    }

    pub fn get_sink(&self) -> &Option<Arc<RwLock<Sink>>> {
        &self.sink
    }

    pub fn set_os(&mut self, os: Option<Arc<OutputStream>>) {
        self.os = os;
    }

    pub fn set_sink(&mut self, sink: Option<Arc<RwLock<Sink>>>) {
        self.sink = sink;
    }

    pub fn pause(&mut self) {
        if let Some(sink) = &self.get_sink() {
            let writeable = sink.write();
            if let Ok(w) = writeable{
                w.pause();
            } else {
                log::error!("No write lock.");
            }
        } else {
            log::error!("No bg sink to manage.");
        }
    }

    pub fn play(&mut self) {
        if let Some(sink) = &self.get_sink() {
            let writeable = sink.write();
            if let Ok(w) = writeable{
                w.play();
            } else {
                log::error!("No write lock.");
            }
        } else {
            log::error!("No bg sink to manage.");
        }
    }

    pub fn configure(&mut self, volume: u32) {
        if let Some(sink) = &self.get_sink() {
            let writeable = sink.write();
            if let Ok(w) = writeable{
                log::info!("Handling background music...");
                let float_volume = (volume as f32 / 100.0) as f32;
                w.set_volume(float_volume);
            } else {
                log::error!("No write lock.");
            }
        } else {
            log::error!("No bg sink to manage.");
        }
    }
}
