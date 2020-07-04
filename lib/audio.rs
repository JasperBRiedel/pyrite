use crate::pyrite_log;
use crate::resources;
use rodio::DeviceTrait;
use std::collections::HashMap;
use std::io::BufReader;

pub struct AudioServer {
    output_device: Option<rodio::Device>,
    tracks: HashMap<String, rodio::Sink>,
}

impl AudioServer {
    pub fn new() -> Self {
        let output_device = rodio::default_output_device();

        match output_device {
            Some(od) => pyrite_log!(
                "Audio server started with \"{}\"",
                od.name().unwrap_or("Unknown".to_string())
            ),
            None => pyrite_log!("Failed to start default audio device"),
        };

        Self {
            output_device: rodio::default_output_device(),
            tracks: HashMap::new(),
        }
    }

    pub fn stop(&mut self, track_name: &str) {
        match self.tracks.get(track_name) {
            Some(track) => track.stop(),
            None => pyrite_log!("Failed to stop track \"{}\": track not found", track_name),
        }
    }

    pub fn stop_all(&mut self) {
        self.tracks.values().for_each(|track| track.stop());
    }

    pub fn play(&mut self, track_name: &str, resources: &Box<dyn resources::Provider>) {
        let output_device = match &self.output_device {
            Some(od) => od,
            None => return,
        };

        // resume the track if it exists and was paused
        if let Some(track) = self.tracks.get(track_name) {
            if track.is_paused() {
                track.play();
                return;
            } else if !track.empty() {
                return;
            }
        }

        let track_data = match resources.read_to_bytes(track_name) {
            Some(td) => td,
            None => {
                pyrite_log!("Audio resource not found \"{}\"", track_name);
                return;
            }
        };

        let track_source =
            match rodio::Decoder::new(BufReader::new(std::io::Cursor::new(track_data))) {
                Ok(ts) => ts,
                Err(e) => {
                    pyrite_log!("Failed to decode audio \"{}\": {}", track_name, e);
                    return;
                }
            };

        let track = rodio::Sink::new(output_device);
        track.append(track_source);
        self.tracks.insert(track_name.to_owned(), track);
    }

    pub fn pause(&mut self, track_name: &str) {
        match self.tracks.get(track_name) {
            Some(track) => track.pause(),
            None => pyrite_log!("Failed to pause track \"{}\": track not found", track_name),
        }
    }

    pub fn volume(&mut self, track_name: &str, value: f32) {
        match self.tracks.get(track_name) {
            Some(track) => track.set_volume(value),
            None => pyrite_log!("Failed to volume track \"{}\": track not found", track_name),
        }
    }
}
