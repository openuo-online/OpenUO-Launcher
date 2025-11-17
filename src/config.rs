use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;


const PROFILES_DIR: &str = "Profiles";
const SETTINGS_DIR: &str = "Profiles/Settings";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherConfig {
    #[serde(skip)]
    pub profiles: Vec<ProfileConfig>,
    #[serde(skip)]
    pub active_profile: usize,
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            profiles: Vec::new(),
            active_profile: 0,
        }
    }
}

// Profile 索引文件结构（Profiles/{uuid}.json）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileIndex {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "SettingsFile")]
    pub settings_file: String,
    #[serde(rename = "FileName")]
    pub file_name: String,
    #[serde(rename = "LastCharacterName")]
    pub last_character_name: String,
    #[serde(rename = "AdditionalArgs")]
    pub additional_args: String,
}

impl Default for ProfileIndex {
    fn default() -> Self {
        Self {
            name: "空白信息".to_string(),
            settings_file: uuid::Uuid::new_v4().to_string(),
            file_name: uuid::Uuid::new_v4().to_string(),
            last_character_name: String::new(),
            additional_args: String::new(),
        }
    }
}

// 运行时使用的完整 Profile 结构
#[derive(Debug, Clone)]
pub struct ProfileConfig {
    pub index: ProfileIndex,
    pub settings: OuoSettings,
}

impl Default for ProfileConfig {
    fn default() -> Self {
        Self {
            index: ProfileIndex::default(),
            settings: OuoSettings::default(),
        }
    }
}

pub fn new_profile(name: &str) -> ProfileConfig {
    let mut profile = ProfileConfig::default();
    profile.index.name = name.to_string();
    profile
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point2 {
    #[serde(rename = "X")]
    pub x: i32,
    #[serde(rename = "Y")]
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
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
    
    // Launcher 添加的 HiDPI/缩放支持参数
    #[serde(rename = "launcher_screen_width", skip_serializing_if = "Option::is_none")]
    pub launcher_screen_width: Option<u32>,
    #[serde(rename = "launcher_screen_height", skip_serializing_if = "Option::is_none")]
    pub launcher_screen_height: Option<u32>,
    #[serde(rename = "launcher_scale_factor", skip_serializing_if = "Option::is_none")]
    pub launcher_scale_factor: Option<f64>,
    #[serde(rename = "launcher_is_hidpi", skip_serializing_if = "Option::is_none")]
    pub launcher_is_hidpi: Option<bool>,
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
            launcher_screen_width: None,
            launcher_screen_height: None,
            launcher_scale_factor: None,
            launcher_is_hidpi: None,
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

pub fn profiles_dir() -> PathBuf {
    base_dir().join(PROFILES_DIR)
}

pub fn settings_dir() -> PathBuf {
    base_dir().join(SETTINGS_DIR)
}

pub fn profile_index_path(profile: &ProfileConfig) -> PathBuf {
    profiles_dir().join(format!("{}.json", profile.index.file_name))
}

pub fn profile_settings_path(profile: &ProfileConfig) -> PathBuf {
    settings_dir().join(format!("{}.json", profile.index.settings_file))
}

// Config loading and saving
pub fn load_config_from_disk() -> LauncherConfig {
    let mut config = LauncherConfig::default();
    
    // 扫描 Profiles 目录加载所有档案
    let profiles_path = profiles_dir();
    fs::create_dir_all(&profiles_path).ok();
    
    let mut profiles = Vec::new();
    
    if let Ok(entries) = fs::read_dir(&profiles_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(profile) = load_profile_from_file(&path) {
                    profiles.push(profile);
                }
            }
        }
    }
    
    // 如果没有档案，创建一个默认档案
    if profiles.is_empty() {
        let default_profile = new_profile("默认配置");
        if save_profile(&default_profile).is_ok() {
            profiles.push(default_profile);
        }
    }
    
    config.profiles = profiles;
    config.active_profile = 0;
    config
}

fn load_profile_from_file(path: &PathBuf) -> Result<ProfileConfig> {
    let raw = fs::read_to_string(path)?;
    let index: ProfileIndex = serde_json::from_str(&raw)?;
    
    tracing::info!("{}: {}", crate::i18n::t!("log.profile_loaded"), index.name);
    
    let mut profile = ProfileConfig {
        index,
        settings: OuoSettings::default(),
    };
    
    // 加载对应的 settings 文件
    let settings_path = profile_settings_path(&profile);
    
    match fs::read_to_string(&settings_path) {
        Ok(settings_raw) => {
            match serde_json::from_str::<OuoSettings>(&settings_raw) {
                Ok(settings) => {
                    tracing::info!("{}: {}", crate::i18n::t!("log.settings_loaded"), settings.username);
                    profile.settings = settings;
                }
                Err(_e) => {
                    tracing::warn!("{}", crate::i18n::t!("log.settings_parse_failed"));
                }
            }
        }
        Err(_e) => {
            tracing::warn!("{}", crate::i18n::t!("log.settings_read_failed"));
        }
    }
    
    Ok(profile)
}

pub fn save_profile(profile: &ProfileConfig) -> Result<()> {
    save_profile_with_screen_info(profile, None)
}

pub fn save_profile_with_screen_info(
    profile: &ProfileConfig,
    screen_info: Option<ScreenInfo>,
) -> Result<()> {
    // 创建必要的目录
    fs::create_dir_all(profiles_dir())?;
    fs::create_dir_all(settings_dir())?;
    
    // 保存索引文件
    let index_json = serde_json::to_string_pretty(&profile.index)?;
    let index_path = profile_index_path(profile);
    let tmp = index_path.with_extension("tmp");
    fs::write(&tmp, index_json)?;
    fs::rename(&tmp, &index_path)?;
    
    // 从文件重新加载 settings，保留游戏可能修改的窗口信息
    let settings_path = profile_settings_path(profile);
    let mut settings = if settings_path.exists() {
        // 如果文件存在，加载它以保留窗口位置等信息
        match fs::read_to_string(&settings_path) {
            Ok(raw) => serde_json::from_str::<OuoSettings>(&raw).unwrap_or_else(|_| profile.settings.clone()),
            Err(_) => profile.settings.clone(),
        }
    } else {
        profile.settings.clone()
    };
    
    // 只更新 Launcher 管理的字段，不覆盖窗口信息
    settings.username = profile.settings.username.clone();
    settings.password = profile.settings.password.clone();
    settings.ip = profile.settings.ip.clone();
    settings.port = profile.settings.port;
    settings.ultima_online_directory = profile.settings.ultima_online_directory.clone();
    settings.save_account = profile.settings.save_account;
    settings.auto_login = profile.settings.auto_login;
    settings.reconnect = profile.settings.reconnect;
    
    // 同步一些必要的字段
    // profilespath 留空，让 OpenUO 使用默认位置（OpenUO/Data/Profiles/）
    settings.profiles_path = String::new();
    settings.last_server_name = settings.ip.clone();
    
    // 添加屏幕信息（如果提供）
    if let Some(info) = screen_info {
        settings.launcher_screen_width = Some(info.width);
        settings.launcher_screen_height = Some(info.height);
        settings.launcher_scale_factor = Some(info.scale_factor);
        settings.launcher_is_hidpi = Some(info.is_hidpi);
    }
    
    // 如果不保存账号，清空用户名和密码
    if !settings.save_account {
        settings.username.clear();
        settings.password.clear();
    }
    
    let settings_json = serde_json::to_string_pretty(&settings)?;
    let tmp = settings_path.with_extension("tmp");
    fs::write(&tmp, settings_json)?;
    fs::rename(&tmp, &settings_path)?;
    
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub struct ScreenInfo {
    pub width: u32,
    pub height: u32,
    pub scale_factor: f64,
    pub is_hidpi: bool,
}

pub fn save_config(config: &LauncherConfig) -> Result<()> {
    // 保存所有档案
    for profile in &config.profiles {
        save_profile(profile)?;
    }
    Ok(())
}

pub fn delete_profile(profile: &ProfileConfig) -> Result<()> {
    let index_path = profile_index_path(profile);
    let settings_path = profile_settings_path(profile);
    
    if index_path.exists() {
        fs::remove_file(index_path)?;
    }
    if settings_path.exists() {
        fs::remove_file(settings_path)?;
    }
    
    Ok(())
}

fn detect_client_version_from_uo_resources(_path: &str) -> Option<String> {
    // TODO: parse client.exe version when available
    None
}
