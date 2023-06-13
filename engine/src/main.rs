use wasmtime::component::*;
use wasmtime::{Config, Engine, Store};
use anyhow::{Result, bail};
use std::borrow::Borrow;
use std::default;
use std::path::Path;
use std::str::FromStr;
use std::time::SystemTime;
use structopt::StructOpt;

bindgen!();

#[derive(StructOpt, Debug, Clone)]
enum Browser {
    Firefox,
    Chrome,
    Edge,
}

impl FromStr for Browser {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "firefox" => Ok(Browser::Firefox),
            "chrome" => Ok(Browser::Chrome),
            "edge" => Ok(Browser::Edge),
            _ => bail!("unknown option"),
        }
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "Recorder Framework")]
struct Args {
    url: String,
    #[structopt(short, long)]
    cookie_from_browser: Option<Browser>,
}


// PluginObject is designed to run in single-thread
struct PluginObject {
    info: recorder_plugin::PluginInfo,
    component: Component,
}

impl PluginObject {
    pub fn from_file<P: AsRef<std::path::Path>, T: Default>(engine: &Engine, linker: &Linker<T>, plugin: P) -> Result<Self> {
        let component = Component::from_file(engine, plugin)?;
        let mut store = Store::new(engine, T::default());
        let (bindings, _) = RecorderPlugin::instantiate(&mut store, &component, linker)?;
        let info = bindings.recorder_plugin().call_get_info(&mut store)?;
        Ok(Self {
            component: component,
            info: info,
        })
    }

    pub fn info(&self) -> &recorder_plugin::PluginInfo {
        &self.info
    }

    pub fn get_instance<T: Default>(&self, engine: &Engine, linker: &Linker<T>) -> Result<PluginInstance<T>> {
        let mut store = Store::new(engine, T::default());
        let (bindings, _) = RecorderPlugin::instantiate(&mut store, &self.component, linker)?;
        Ok(PluginInstance {
            store, bindings,
        })
    }
}

struct PluginInstance<T> {
    store: Store<T>,
    bindings: RecorderPlugin,
}

use recorder_plugin::*;
impl<T> PluginInstance<T> {
    pub fn get_info(&mut self) -> Result<PluginInfo> {
        self.bindings.recorder_plugin().call_get_info(&mut self.store)
    }

    pub fn match_url<S: AsRef<str>>(&mut self, url: S) -> Result<Result<MatchResult, Error>> {
        self.bindings.recorder_plugin().call_match_url(&mut self.store, url.as_ref())
    }
} 

struct PluginManager<T> {
    engine: Engine,
    linker: Linker<T>,
    plugins: Vec<PluginObject>,
}

impl<T: Default> PluginManager<T> {
    fn new() -> Result<Self> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        log::trace!("create wasm engine.");
        let engine = Engine::new(&config)?;
        let linker = Linker::new(&engine);
        Ok(PluginManager { engine: engine, linker: linker, plugins: Vec::new() })
    }

    fn load_plugins<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        if path.is_dir() {
            log::debug!("handle path {}", path.to_string_lossy());
            for f in path.read_dir()? {
                match f.map_err(|e| e.into()).and_then(|f| self.load_plugins(f.path())) {
                    Ok(_)=> { },
                    Err(e) => {
                        log::warn!("error while reading directory: {}", e);
                    }
                }
            }
        } else if path.is_file() && path.extension().map(|e| (e == "wasm")) == Some(true) {
            match PluginObject::from_file(&self.engine, &self.linker, &path ) {
                Ok(plugin) => {
                    log::info!("plugin info {:?}", plugin.info());
                    self.plugins.push(plugin);
                    log::info!("plugin {} loaded", path.to_string_lossy());
                },
                Err(e) => {
                    log::warn!("error while loading plugin {}, {}", path.to_string_lossy(), e)
                }
            }
        }
        Ok(())
    }

    fn plugin_count(&self) -> usize {
        self.plugins.len()
    }
}


struct App{ }
impl Default for App {
    fn default() -> Self {
        App{}
    }
}

struct Server<T> {
    pm: PluginManager<T>,
}

impl<T: Default> Server<T> {
    fn new() -> Result<Self> {
        let mut pm = PluginManager::<T>::new()?;
        let plugin_dir = std::env::current_dir().unwrap_or(std::path::PathBuf::from_str(".")?);
        //plugin_dir.push("plugins");
        log::info!("load plugins.");
        pm.load_plugins(plugin_dir)?;
        log::info!("{} plugins loaded.", pm.plugin_count());
        if pm.plugin_count() < 1 { bail!("no plugin exists") }
        Ok(Server {
            pm: pm,
        })
    }
}


fn main() -> wasmtime::Result<()> {
    setup_logger()?;
    let args = Args::from_args();
    let server = Server::<App>::new();
    Ok(())
}


fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .chain(fern::log_file("log")?)
        .apply()?;
    Ok(())
}
