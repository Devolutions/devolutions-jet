use dlopen::{symbor::Library, Error};
use lazy_static::lazy_static;
use std::sync::{Mutex, Arc};
use slog_scope::debug;

mod general;
mod packets_parsing;
mod recording;
use general::{PluginCapabilities, PluginInformation};
pub use recording::Recorder;
pub use packets_parsing::PacketsParser;

pub struct PluginManager {
    lib: Vec<Arc<Library>>,
}

lazy_static! {
   pub static ref PLUGIN_MANAGER: Mutex<PluginManager> = Mutex::new(PluginManager {
            lib: Vec::new(),
        });
}

impl PluginManager {

    pub fn get_recording_plugin(&self) -> Option<Recorder> {
        for lib in &self.lib {
            let info = PluginInformation::new(lib.clone());
            return if info.get_capabilities().contains(&PluginCapabilities::Recording) {
                debug!("recording plugin found");
                Some(Recorder::new(lib.clone()))
            } else {
                None
            }
        }
        None
    }

    pub fn get_parsing_packets_plugin(&self) -> Option<PacketsParser> {
        for lib in &self.lib {
            let info = PluginInformation::new(lib.clone());
            return if info.get_capabilities().contains(&PluginCapabilities::PacketsParsing) {
                debug!("parsing plugin found");
                Some(PacketsParser::new(lib.clone()))
            } else {
                None
            }
        }
        None
    }

    pub fn load_plugin(&mut self, path: &String) -> Result<(), Error> {
        let lib = Arc::new(Library::open(path)?);
        self.lib.push(lib.clone());
        let info = PluginInformation::new(lib.clone());
        slog_scope::info!("Plugin {} loaded", info.get_name());
        Ok(())
    }
}