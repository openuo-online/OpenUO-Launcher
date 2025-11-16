use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::warn;

const CONFIG_FILE: &str = "launcher_state.json";
const PROFILES_DIR: &str = "Profiles";
const SETTINGS_DIR: &str = "Profiles/Settings";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherConfig {
    #[serde(default = "default_profiles")]
    pub profiles: Vec<ProfileConfig>,
    #[serde(default)]
    pub active_profile: usize,
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            profiles: default_profiles(),
            active_profile: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    pub name: String,
    pub last_character: String,
    pub additional_args: String,
    pub settings_file: String,
    #[serde(default)]
    pub settings: OuoSettings,
}

impl Default for ProfileConfig {
    fn default() -> Self {
        new_profile("默认配置")
    }
}

fn default_profiles() -> Vec<ProfileConfig> {
    vec![ProfileConfig::default()]
}

pub fn new_profile(name: &str) -> ProfileConfig {
    ProfileConfig {
        name: name.to_string(),
        last_character: String::new(),
        additional_args: String::new(),
        settings_file: uuid::Uuid::new_v4().to_string(),
        settings: OuoSettings::default(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point2 {
    #[serde(rename = "X")]
    pub x: i32,
    #[serde(rename = "Y")]
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OuoSettings {
    #[serde(rename = "username")]
    pub username: String,
    #[serde(rename = "password")]
    pub password: String,
    #[serde(rename = "ip")]
    pub ip: String,
    #[serde(rename = "port")]
    pub port: u16,
    #[serde(rename = "ultimaonlinedirectory")]
    pub ultima_online_directory: String,
    #[serde(rename = "profilespath")]
    pub profiles_path: String,
    #[serde(rename = "clientversion")]
    pub client_version: String,
    #[serde(rename = "lang")]
    pub language: String,
    #[serde(rename = "lastservernum")]
    pub last_server_num: u16,
    #[serde(rename = "last_server_name")]
    pub last_server_name: String,
    #[serde(rename = "fps")]
    pub fps: i32,
    #[serde(rename = "window_position")]
    pub window_position: Option<Point2>,
    #[serde(rename = "window_size")]
    pub window_size: Option<Point2>,
    #[serde(rename = "is_win_maximized")]
    pub is_window_maximized: bool,
    #[serde(rename = "saveaccount")]
    pub save_account: bool,
    #[serde(rename = "autologin")]
    pub auto_login: bool,
    #[serde(rename = "reconnect")]
    pub reconnect: bool,
    #[serde(rename = "reconnect_time")]
    pub reconnect_time: i32,
    #[serde(rename = "login_music")]
    pub login_music: bool,
    #[serde(rename = "login_music_volume")]
    pub login_music_volume: i32,
    #[serde(rename = "shard_type")]
    pub shard_type: i32,
    #[serde(rename = "fixed_time_step")]
    pub fixed_time_step: bool,
    #[serde(rename = "run_mouse_in_separate_thread")]
    pub run_mouse_in_separate_thread: bool,
    #[serde(rename = "force_driver")]
    pub force_driver: u8,
    #[serde(rename = "use_verdata")]
    pub use_verdata: bool,
    #[serde(rename = "maps_layouts")]
    pub maps_layouts: String,
    #[serde(rename = "encryption")]
    pub encryption: u8,
    #[serde(rename = "plugins")]
    pub plugins: Vec<String>,
}

impl Default for OuoSettings {
    fn default() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            ip: "openuo.online".to_string(),
            port: 2593,
            ultima_online_directory: String::new(),
            profiles_path: String::new(),
            client_version: String::new(),
            language: String::new(),
            last_server_num: 1,
            last_server_name: String::new(),
            fps: 60,
            window_position: None,
            window_size: None,
            is_window_maximized: true,
            save_account: true,
            auto_login: true,
            reconnect: true,
            reconnect_time: 1,
            login_music: true,
            login_music_volume: 70,
            shard_type: 0,
            fixed_time_step: true,
            run_mouse_in_separate_thread: true,
            force_driver: 0,
            use_verdata: false,
            maps_layouts: String::new(),
            encryption: 0,
            plugins: Vec::new(),
        }
    }
}

// Path helpers
pub fn client_path() -> String {
    "OpenUO".to_string()
}

pub fn uo_data_path() -> String {
    client_path()
}

fn base_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

pub fn open_uo_dir() -> PathBuf {
    let path = client_path();
    if PathBuf::from(&path).is_absolute() {
        PathBuf::from(&path)
    } else {
        base_dir().join(&path)
    }
}

pub fn open_uo_binary_path() -> PathBuf {
    let dir = open_uo_dir();
    let exe = if cfg!(target_os = "windows") {
        "OpenUO.exe"
    } else {
        "OpenUO"
    };
    dir.join(exe)
}

pub fn uo_data_dir_path() -> PathBuf {
    open_uo_dir()
}

fn config_path() -> PathBuf {
    base_dir().join(CONFIG_FILE)
}

pub fn profiles_dir() -> PathBuf {
    base_dir().join(PROFILES_DIR)
}

pub fn settings_dir() -> PathBuf {
    base_dir().join(SETTINGS_DIR)
}

pub fn profile_settings_path(profile: &ProfileConfig) -> PathBuf {
    settings_dir().join(format!("{}.json", profile.settings_file))
}

// Config loading and saving
pub fn load_config_from_disk() -> LauncherConfig {
    let path = config_path();
    let mut cfg = match fs::read_to_string(&path) {
        Ok(raw) => match serde_json::from_str::<LauncherConfig>(&raw) {
            Ok(mut cfg) => {
                cfg.active_profile = cfg.active_profile.min(cfg.profiles.len().saturating_sub(1));
                cfg
            }
            Err(err) => {
                warn!("配置文件解析失败 {:?}: {err}", path);
                LauncherConfig::default()
            }
        },
        Err(_) => LauncherConfig::default(),
    };

    for profile in &mut cfg.profiles {
        hydrate_profile_defaults(profile);
        load_profile_settings(profile);
    }

    cfg
}

pub fn save_config(config: &mut LauncherConfig) -> Result<PathBuf> {
    fs::create_dir_all(settings_dir())?;

    let client_path_str = open_uo_dir().to_string_lossy().to_string();
    let uo_data_path_str = uo_data_dir_path().to_string_lossy().to_string();
    let profiles_path = profiles_dir().to_string_lossy().to_string();

    for profile in &mut config.profiles {
        hydrate_profile_defaults(profile);
        sync_profile_into_settings(profile, &client_path_str, &uo_data_path_str, &profiles_path);
        write_profile_settings(profile)?;
    }

    let mut sanitized = config.clone();
    for profile in &mut sanitized.profiles {
        if !profile.settings.save_account {
            profile.settings.username.clear();
            profile.settings.password.clear();
        }
    }

    let data = serde_json::to_string_pretty(&sanitized)?;
    let path = config_path();
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, data)?;
    fs::rename(&tmp, &path)?;
    Ok(path)
}

fn hydrate_profile_defaults(profile: &mut ProfileConfig) {
    if profile.settings_file.is_empty() {
        profile.settings_file = uuid::Uuid::new_v4().to_string();
    }
}

fn sync_profile_into_settings(
    profile: &mut ProfileConfig,
    _open_uo_dir: &str,
    _uo_data_path: &str,
    profiles_path: &str,
) {
    // 不要自动覆盖用户设置的 ultima_online_directory
    // 只在为空时才设置默认值
    if profile.settings.ultima_online_directory.is_empty() {
        profile.settings.ultima_online_directory = String::new();
    }
    profile.settings.profiles_path = profiles_path.to_string();
    profile.settings.last_server_name = profile.settings.ip.clone();
    profile.settings.client_version =
        detect_client_version_from_uo_resources(&profile.settings.ultima_online_directory)
            .unwrap_or_default();
    if !profile.settings.save_account {
        profile.settings.username.clear();
        profile.settings.password.clear();
    }
}

fn load_profile_settings(profile: &mut ProfileConfig) {
    let path = profile_settings_path(profile);
    if let Ok(raw) = fs::read_to_string(&path) {
        if let Ok(settings) = serde_json::from_str::<OuoSettings>(&raw) {
            profile.settings = settings;
            return;
        }
    }
    profile.settings = OuoSettings::default();
}

fn write_profile_settings(profile: &ProfileConfig) -> Result<()> {
    let mut data = profile.settings.clone();
    if !data.save_account {
        data.username.clear();
        data.password.clear();
    }
    let json = serde_json::to_string_pretty(&data)?;
    let path = profile_settings_path(profile);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, json)?;
    fs::rename(&tmp, &path)?;
    Ok(())
}

fn detect_client_version_from_uo_resources(_path: &str) -> Option<String> {
    // TODO: parse client.exe version when available
    None
}
