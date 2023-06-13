wit_bindgen::generate!("recorder-plugin");
use recorder_plugin::{RecorderPlugin, MatchResult, PluginInfo, Error, Version};

struct DummyPlugin;

impl RecorderPlugin for DummyPlugin {
    fn get_info() -> PluginInfo {
        PluginInfo {
            name: "DummyPlugin".to_string(),
            author: Some("Dummy".to_string()),
            description: Some("Demo of how to write a plugin".to_string()),
            version: Some(Version { major: 0, minor: 0, build: 0}),
            arguments: None,
        }

    }

    fn match_url(url: String) -> Result<MatchResult, Error>{
        log::trace!("url: {}", url);
        return Ok(MatchResult::Video);
    }
}

export_recorder_plugin!(DummyPlugin);
