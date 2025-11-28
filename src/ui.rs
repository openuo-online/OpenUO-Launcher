use anyhow::{Context, Result};
use egui::{Color32, ColorImage, RichText};
use std::process::Command;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::config::*;
use crate::github::*;
use crate::i18n::t;
use crate::profile_editor::ProfileEditor;

/// 日志条目类型
#[derive(Debug, Clone)]
pub enum LogEntryType {
    Info,
    Success,
    Warning,
    Error,
    Checking,
}

/// 日志条目
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: Instant,
    pub entry_type: LogEntryType,
    pub message: String,
    pub action: Option<LogAction>,
}

/// 日志关联的操作
#[derive(Debug, Clone)]
pub enum LogAction {
    UpdateLauncher,
    UpdateOpenUO,
    RetryDownload,
}

pub struct LauncherUi {
    pub config: LauncherConfig,
    pub profile_editor: ProfileEditor,
    pub open_uo_version: Option<String>,
    pub launcher_version: String,
    pub download_rx: Option<mpsc::Receiver<DownloadEvent>>,
    pub download_progress: Option<(u64, u64)>,
    pub downloading_launcher: bool,
    pub launcher_restarting: bool,
    pub update_rx: Option<mpsc::Receiver<UpdateEvent>>,
    pub remote_open_uo: Option<String>,
    pub remote_launcher: Option<String>,
    pub last_update_poll: Instant,
    pub checking_open_uo: bool,
    pub checking_launcher: bool,
    pub background_texture: Option<egui::TextureHandle>,
    pub logo_texture: Option<egui::TextureHandle>,
    pub screen_info: Option<ScreenInfo>,
    pub current_locale: String,
    pub logs: Vec<LogEntry>,
    pub download_failed: bool,
}

impl LauncherUi {
    pub fn new(config: LauncherConfig) -> Self {
        Self {
            config,
            profile_editor: ProfileEditor::new(),
            open_uo_version: detect_open_uo_version(),
            launcher_version: format!("v{}", env!("CARGO_PKG_VERSION")),
            download_rx: None,
            download_progress: None,
            downloading_launcher: false,
            launcher_restarting: false,
            update_rx: None,
            remote_open_uo: None,
            screen_info: None,
            remote_launcher: None,
            last_update_poll: Instant::now() - Duration::from_secs(601),
            checking_open_uo: false,
            checking_launcher: false,
            background_texture: None,
            logo_texture: None,
            current_locale: crate::i18n::current_locale().to_string(),
            logs: Vec::new(),
            download_failed: false,
        }
    }

    pub fn ui(&mut self, ctx: &egui::Context) {
        self.poll_channels();
        self.maybe_schedule_updates();
        self.ensure_textures(ctx);

        // Global visuals: keep panels transparent
        {
            let mut style = (*ctx.style()).clone();
            style.visuals.window_fill = Color32::TRANSPARENT;
            style.visuals.panel_fill = Color32::TRANSPARENT;
            ctx.set_style(style);
        }

        self.show_profile_editor(ctx);
        self.show_main_panel(ctx);
    }

    fn show_profile_editor(&mut self, ctx: &egui::Context) {
        if let Some((idx, mut profile)) = self.profile_editor.show(ctx) {
            // 加密密码后再保存
            profile.settings.password = crate::crypter::encrypt(&profile.settings.password);
            self.config.profiles[idx] = profile;
            self.config.active_profile = idx;
            // 保存配置到文件（带屏幕信息）
            match self.save_config_with_screen_info() {
                Ok(_) => self.set_status(&t!("status.config_saved")),
                Err(_err) => self.set_status(&t!("status.save_failed")),
            }
        }
    }

    fn show_main_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::TRANSPARENT))
            .show(ctx, |ui| {
                ui.visuals_mut().widgets.inactive.bg_fill = Color32::TRANSPARENT;
                ui.visuals_mut().widgets.noninteractive.bg_fill = Color32::TRANSPARENT;
                ui.visuals_mut().widgets.active.bg_fill = Color32::TRANSPARENT;
                ui.visuals_mut().widgets.hovered.bg_fill = Color32::TRANSPARENT;
                ui.visuals_mut().widgets.open.bg_fill = Color32::TRANSPARENT;

                paint_background(ui, &self.background_texture, &self.logo_texture);
                
                let margin = 12.0;
                let available_rect = ui.available_rect_before_wrap();
                let footer_height = 30.0;
                
                // 主内容区域
                let content_rect = egui::Rect::from_min_size(
                    available_rect.min,
                    egui::vec2(available_rect.width(), available_rect.height() - footer_height)
                );
                
                let mut content_ui = ui.child_ui(content_rect, egui::Layout::top_down(egui::Align::Min));
                content_ui.add_space(margin);
                
                content_ui.horizontal(|ui| {
                    ui.add_space(margin);
                    
                    ui.vertical(|ui| {
                        // 标题
                        ui.heading(RichText::new(t!("window.title")).size(24.0).strong());
                        ui.add_space(12.0);

                        // 语言选择
                        self.show_language_selector(ui);
                        ui.add_space(8.0);
                        
                        // 配置选择
                        self.show_profile_selector(ui);
                        ui.add_space(8.0);
                        
                        // 启动按钮
                        self.show_launch_button(ui);
                        ui.add_space(12.0);
                        
                        // 日志区域
                        self.show_log_area(ui);
                    });
                    
                    ui.add_space(margin);
                });
                
                // 底部信息栏（固定在底部）
                let footer_rect = egui::Rect::from_min_size(
                    egui::pos2(available_rect.min.x, available_rect.max.y - footer_height),
                    egui::vec2(available_rect.width(), footer_height)
                );
                
                let mut footer_ui = ui.child_ui(footer_rect, egui::Layout::top_down(egui::Align::Min));
                self.show_footer(&mut footer_ui);
            });
    }

    fn show_language_selector(&mut self, ui: &mut egui::Ui) {
        egui::Frame::none().show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(t!("main.language"));
                
                // 获取可用语言列表
                let languages = crate::i18n::available_languages();
                
                // 查找当前语言的显示名称
                let current_name = languages
                    .iter()
                    .find(|lang| lang.code == self.current_locale)
                    .map(|lang| lang.native_name.as_str())
                    .unwrap_or(&self.current_locale);
                
                egui::ComboBox::from_id_source("language_combo")
                    .selected_text(current_name)
                    .show_ui(ui, |ui| {
                        // 动态生成语言选项
                        for lang in languages {
                            let is_selected = self.current_locale == lang.code;
                            if ui.selectable_label(is_selected, &lang.native_name).clicked() {
                                self.current_locale = lang.code.clone();
                                crate::i18n::set_locale(&lang.code);
                                
                                // 保存用户选择的语言
                                self.config.launcher_settings.language = Some(lang.code.clone());
                                if let Err(e) = save_launcher_settings(&self.config.launcher_settings) {
                                    tracing::warn!("Failed to save language setting: {}", e);
                                }
                            }
                        }
                    });
            });
        });
    }

    fn show_profile_selector(&mut self, ui: &mut egui::Ui) {
        egui::Frame::none().show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(t!("main.profile"));
                let profile_name = self
                    .active_profile()
                    .map(|p| p.index.name.as_str())
                    .unwrap_or("");

                egui::ComboBox::from_id_source("profile_combo")
                    .selected_text(profile_name)
                    .show_ui(ui, |ui| {
                        for (idx, profile) in self.config.profiles.iter().enumerate() {
                            let selected = idx == self.config.active_profile;
                            if ui.selectable_label(selected, &profile.index.name).clicked() {
                                self.config.active_profile = idx;
                            }
                        }
                    });

                let edit_btn = egui::Button::new(t!("main.edit"))
                    .fill(egui::Color32::from_rgba_unmultiplied(50, 120, 200, 200))
                    .min_size(egui::vec2(60.0, 24.0));
                if ui.add(edit_btn).clicked() {
                    self.open_profile_editor();
                }
                
                let new_btn = egui::Button::new(t!("main.new"))
                    .fill(egui::Color32::from_rgba_unmultiplied(50, 180, 100, 200))
                    .min_size(egui::vec2(60.0, 24.0));
                if ui.add(new_btn).clicked() {
                    self.add_profile();
                }
                
                let copy_btn = egui::Button::new(t!("main.copy"))
                    .fill(egui::Color32::from_rgba_unmultiplied(100, 150, 200, 200))
                    .min_size(egui::vec2(60.0, 24.0));
                if ui.add(copy_btn).clicked() {
                    self.duplicate_profile();
                }
                
                let delete_btn = egui::Button::new(t!("main.delete"))
                    .fill(egui::Color32::from_rgba_unmultiplied(200, 80, 80, 200))
                    .min_size(egui::vec2(60.0, 24.0));
                if ui.add(delete_btn).clicked() {
                    self.delete_profile();
                }
            });
        });
    }

    fn show_version_info(&mut self, ui: &mut egui::Ui) {
        egui::Frame::none().show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            let launcher_remote = if self.checking_launcher {
                t!("version.checking").to_string()
            } else {
                self.remote_launcher.clone().unwrap_or_else(|| t!("version.check_failed").to_string())
            };
            let launcher_version = self.launcher_version.clone();
            let has_update = self.remote_launcher.as_ref()
                .map(|r| r != &launcher_version && !self.checking_launcher)
                .unwrap_or(false);
            
            ui.horizontal(|ui| {
                ui.label(format!(
                    "{} {}  {}: {}",
                    t!("version.launcher_local"), launcher_version,
                    t!("version.launcher_remote"), launcher_remote
                ));
                
                // 检查是否有新版本或正在下载或正在重启
                if has_update || self.downloading_launcher || self.launcher_restarting {
                    let is_busy = self.downloading_launcher || self.launcher_restarting;
                    let btn_text = if self.launcher_restarting {
                        t!("version.restarting").to_string()
                    } else if self.downloading_launcher {
                        t!("version.updating").to_string()
                    } else {
                        t!("version.update_launcher").to_string()
                    };
                    
                    let btn_color = if is_busy {
                        egui::Color32::from_rgba_unmultiplied(100, 100, 100, 200)
                    } else {
                        egui::Color32::from_rgba_unmultiplied(200, 100, 50, 200)
                    };
                    
                    let mut update_btn = egui::Button::new(btn_text)
                        .fill(btn_color)
                        .min_size(egui::vec2(100.0, 24.0));
                    
                    // 下载中或重启中时禁用按钮
                    if is_busy {
                        update_btn = update_btn.sense(egui::Sense::hover());
                    }
                    
                    if ui.add(update_btn).clicked() && !is_busy {
                        self.start_launcher_update();
                    }
                    
                    // 显示下载进度（仅当正在下载 Launcher 时）
                    if self.downloading_launcher {
                        if let Some((cur, total)) = self.download_progress {
                            if total > 0 {
                                let progress = (cur as f32) / (total as f32);
                                let total_mb = (total as f32) / (1024.0 * 1024.0);
                                let cur_mb = (cur as f32) / (1024.0 * 1024.0);
                                
                                ui.add(
                                    egui::ProgressBar::new(progress)
                                        .text(format!("{:.1}/{:.1} MB", cur_mb, total_mb))
                                        .desired_width(150.0)
                                );
                            }
                        }
                    }
                }
            });
            
            ui.horizontal(|ui| {
                let open_uo_text = self
                    .open_uo_version
                    .clone()
                    .unwrap_or_else(|| t!("version.not_installed").to_string());
                let remote = if self.checking_open_uo {
                    t!("version.checking").to_string()
                } else {
                    self.remote_open_uo.as_deref().map(|s| s.to_string()).unwrap_or_else(|| t!("version.check_failed").to_string())
                };
                ui.label(format!("{} {}  {}: {}", 
                    t!("version.openuo_local"), open_uo_text,
                    t!("version.openuo_remote"), remote
                ));
                
                // 判断是否需要显示下载/更新按钮
                let has_openuo_update = self.remote_open_uo.as_ref()
                    .and_then(|remote| self.open_uo_version.as_ref().map(|local| remote != local))
                    .unwrap_or(false);
                
                let is_downloading_openuo = !self.downloading_launcher && self.download_rx.is_some();
                
                if self.open_uo_version.is_none() || has_openuo_update || is_downloading_openuo {
                    let (btn_text, btn_color) = if is_downloading_openuo {
                        (t!("version.downloading").to_string(), egui::Color32::from_rgba_unmultiplied(100, 100, 100, 200))
                    } else if self.open_uo_version.is_none() {
                        (t!("version.download_openuo").to_string(), egui::Color32::from_rgba_unmultiplied(50, 180, 100, 200))
                    } else {
                        (t!("version.update_openuo").to_string(), egui::Color32::from_rgba_unmultiplied(100, 150, 200, 200))
                    };
                    
                    let mut btn = egui::Button::new(btn_text)
                        .fill(btn_color)
                        .min_size(egui::vec2(100.0, 24.0));
                    
                    // 下载中时禁用按钮
                    if is_downloading_openuo {
                        btn = btn.sense(egui::Sense::hover());
                    }
                    
                    if ui.add(btn).clicked() && !is_downloading_openuo {
                        self.start_download();
                    }
                }
                
                // 显示下载进度（仅当正在下载 OpenUO 时）
                if !self.downloading_launcher && self.download_rx.is_some() {
                    if let Some((cur, total)) = self.download_progress {
                        if total > 0 {
                            let progress = (cur as f32) / (total as f32);
                            let total_mb = (total as f32) / (1024.0 * 1024.0);
                            let cur_mb = (cur as f32) / (1024.0 * 1024.0);
                            
                            ui.add(
                                egui::ProgressBar::new(progress)
                                    .text(format!("{:.1}/{:.1} MB", cur_mb, total_mb))
                                    .desired_width(150.0)
                            );
                        }
                    }
                }
                // 版本一致时不显示任何按钮
            });
        });
    }

    fn show_launch_button(&mut self, ui: &mut egui::Ui) {
        egui::Frame::none().show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                let launch_btn = egui::Button::new(
                    RichText::new(t!("main.launch")).size(18.0).strong()
                )
                .fill(egui::Color32::from_rgba_unmultiplied(80, 180, 80, 220))
                .min_size(egui::vec2(150.0, 40.0));
                
                if ui.add(launch_btn).clicked() {
                    match self.launch_open_uo() {
                        Ok(msg) => self.add_log(LogEntryType::Success, &msg, None),
                        Err(err) => self.add_log(LogEntryType::Error, &format!("✗ {}: {}", t!("status.launch_failed"), err), None),
                    }
                }
            });
        });
    }

    fn show_footer(&mut self, ui: &mut egui::Ui) {
        // 添加半透明背景
        let footer_frame = egui::Frame::none()
            .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 120))
            .inner_margin(egui::Margin::symmetric(12.0, 6.0));
        
        footer_frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                // 左侧：OpenUO 版本
                let openuo_version = self.open_uo_version.as_deref().unwrap_or("N/A");
                ui.label(
                    RichText::new(format!("OpenUO: {}", openuo_version))
                        .size(11.0)
                        .color(egui::Color32::from_rgb(180, 180, 180))
                );
                
                ui.separator();
                
                // 中间：语言和操作系统
                let system_info = crate::system_info::system_info_string();
                let languages = crate::i18n::available_languages();
                let current_lang = languages
                    .iter()
                    .find(|lang| lang.code == self.current_locale)
                    .map(|lang| lang.native_name.as_str())
                    .unwrap_or(&self.current_locale);
                
                ui.label(
                    RichText::new(format!("{} | {}", current_lang, system_info))
                        .size(11.0)
                        .color(egui::Color32::from_rgb(160, 160, 160))
                );
                
                // 右侧：Launcher 版本
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("Launcher: {}", self.launcher_version))
                            .size(11.0)
                            .color(egui::Color32::from_rgb(180, 180, 180))
                    );
                });
            });
        });
    }

    fn poll_channels(&mut self) {
        // 处理下载事件
        if let Some(rx) = &self.download_rx {
            let events: Vec<_> = rx.try_iter().collect();
            for event in events {
                match event {
                    DownloadEvent::Progress { received, total } => {
                        self.download_progress = Some((received, total));
                    }
                    DownloadEvent::Finished(result) => {
                        self.download_rx = None;
                        self.download_progress = None;
                        
                        match result {
                            Ok(tag) => {
                                if tag.starts_with("UPDATE_AND_RESTART:") {
                                    let version = tag.strip_prefix("UPDATE_AND_RESTART:").unwrap_or("");
                                    self.add_log(LogEntryType::Success, &format!("✅ {}", t!("log.launcher_update_complete", version = version)), None);
                                    self.launcher_restarting = true;
                                    std::thread::spawn(|| {
                                        std::thread::sleep(std::time::Duration::from_secs(2));
                                        std::process::exit(0);
                                    });
                                } else {
                                    self.open_uo_version = Some(tag.clone());
                                    self.add_log(LogEntryType::Success, &format!("✓ {}", t!("log.openuo_download_complete", version = &tag)), None);
                                }
                                self.downloading_launcher = false;
                                self.download_failed = false;
                            }
                            Err(err) => {
                                self.add_log(LogEntryType::Error, &format!("✗ {}: {}", t!("log.download_error"), err), Some(LogAction::RetryDownload));
                                self.downloading_launcher = false;
                                self.download_failed = true;
                            }
                        }
                    }
                }
            }
        }
        
        // 处理更新检查事件
        if let Some(rx) = &self.update_rx {
            let events: Vec<_> = rx.try_iter().collect();
            for event in events {
                match event {
                    UpdateEvent::OpenUO(res) => {
                        self.checking_open_uo = false;
                        match res {
                            Ok(v) => {
                                self.remote_open_uo = Some(v.clone());
                                if let Some(local) = &self.open_uo_version {
                                    if &v != local {
                                        self.add_log(LogEntryType::Info, &format!("{}: {}", t!("log.openuo_new_version"), v), Some(LogAction::UpdateOpenUO));
                                    } else {
                                        self.add_log(LogEntryType::Success, &format!("✓ {}: {}", t!("log.openuo_latest"), v), None);
                                    }
                                } else {
                                    self.add_log(LogEntryType::Info, &format!("{}: {}", t!("log.openuo_not_installed"), v), Some(LogAction::UpdateOpenUO));
                                }
                            }
                            Err(e) => {
                                self.add_log(LogEntryType::Error, &format!("✗ {}: {}", t!("log.openuo_check_error"), e), None);
                            }
                        }
                    }
                    UpdateEvent::Launcher(res) => {
                        self.checking_launcher = false;
                        match res {
                            Ok(v) => {
                                self.remote_launcher = Some(v.clone());
                                if v != self.launcher_version {
                                    self.add_log(LogEntryType::Info, &format!("{}: {}", t!("log.launcher_new_version"), v), Some(LogAction::UpdateLauncher));
                                } else {
                                    self.add_log(LogEntryType::Success, &format!("✓ {}: {}", t!("log.launcher_latest"), v), None);
                                }
                            }
                            Err(e) => {
                                self.add_log(LogEntryType::Error, &format!("✗ {}: {}", t!("log.launcher_check_error"), e), None);
                            }
                        }
                    }
                    UpdateEvent::Done => {}
                }
            }
        }
    }

    fn start_download(&mut self) {
        if self.download_rx.is_some() {
            return;
        }
        self.add_log(LogEntryType::Info, &format!("⏳ {}", t!("log.downloading_openuo")), None);
        let (tx, rx) = mpsc::channel();
        let tx_progress = tx.clone();
        std::thread::spawn(move || {
            let result = download_and_unpack_open_uo_with_progress(move |evt| {
                let _ = tx_progress.send(evt);
            });
            let _ = tx.send(DownloadEvent::Finished(result.map_err(|e| format!("{e:#}"))));
        });
        self.download_rx = Some(rx);
        self.download_progress = None;
        self.downloading_launcher = false;
    }

    fn start_launcher_update(&mut self) {
        if self.download_rx.is_some() {
            return;
        }
        self.add_log(LogEntryType::Info, &format!("⏳ {}", t!("log.downloading_launcher")), None);
        let (tx, rx) = mpsc::channel();
        let tx_progress = tx.clone();
        std::thread::spawn(move || {
            let result = crate::github::download_launcher_update(move |evt| {
                let _ = tx_progress.send(evt);
            });
            let _ = tx.send(DownloadEvent::Finished(result.map_err(|e| format!("{e:#}"))));
        });
        self.download_rx = Some(rx);
        self.download_progress = None;
        self.downloading_launcher = true;
    }

    fn trigger_update_checks(&mut self, open_uo: bool, launcher: bool) {
        if !open_uo && !launcher {
            return;
        }
        if open_uo && !self.checking_open_uo {
            self.checking_open_uo = true;
            self.add_log(LogEntryType::Checking, &format!("⟳ {}", t!("log.checking_openuo")), None);
        }
        if launcher && !self.checking_launcher {
            self.checking_launcher = true;
            self.add_log(LogEntryType::Checking, &format!("⟳ {}", t!("log.checking_launcher")), None);
        }
        self.last_update_poll = Instant::now();
        self.update_rx = Some(trigger_update_check_impl(open_uo, launcher));
    }

    fn maybe_schedule_updates(&mut self) {
        if self.checking_open_uo || self.checking_launcher {
            return;
        }
        if self.last_update_poll.elapsed() > Duration::from_secs(600) {
            self.trigger_update_checks(true, true);
        }
    }

    fn ensure_textures(&mut self, ctx: &egui::Context) {
        if self.background_texture.is_none() {
            self.background_texture = load_embedded_texture(
                ctx,
                "launcher_background",
                include_bytes!("../assets/background.png"),
            );
        }
        if self.logo_texture.is_none() {
            self.logo_texture = load_embedded_texture(
                ctx,
                "launcher_logo",
                include_bytes!("../assets/logo.png"),
            );
        }
    }

    fn launch_open_uo(&mut self) -> Result<String> {
        let Some(profile) = self.active_profile().cloned() else {
            anyhow::bail!("{}", t!("status.no_profile"));
        };
        // 保存配置时带上屏幕信息
        self.save_config_with_screen_info()?;
        let settings_path = profile_settings_path(&profile);
        let exe = open_uo_binary_path();
        if !exe.exists() {
            anyhow::bail!("{}", t!("status.openuo_not_found"));
        }

        let mut cmd = Command::new(exe);
        cmd.current_dir(open_uo_dir());
        cmd.arg("-settings")
            .arg(settings_path)
            .arg("-skipupdatecheck");

        if profile.settings.auto_login {
            cmd.arg("-skiploginscreen");
            if !profile.index.last_character_name.is_empty() {
                let last = profile.index.last_character_name.clone();
                cmd.arg("-lastcharactername").arg(last);
            }
        }
        if !profile.index.additional_args.is_empty() {
            cmd.args(profile.index.additional_args.split_whitespace());
        }

        cmd.spawn()
            .with_context(|| t!("status.launch_failed").to_string())?;

        Ok(t!("status.launch_success").to_string())
    }

    fn active_profile(&self) -> Option<&ProfileConfig> {
        self.config.profiles.get(self.config.active_profile)
    }

    fn open_profile_editor(&mut self) {
        if let Some(profile) = self.active_profile().cloned() {
            let idx = self.config.active_profile;
            self.profile_editor.open(profile, idx);
        }
    }

    fn add_profile(&mut self) {
        let p = new_profile(&format!("{} {}", t!("main.profile"), self.config.profiles.len() + 1));
        self.config.profiles.push(p);
        self.config.active_profile = self.config.profiles.len().saturating_sub(1);
        self.set_status(&t!("status.profile_added"));
    }

    fn duplicate_profile(&mut self) {
        if let Some(profile) = self.active_profile().cloned() {
            let mut cloned = profile;
            cloned.index.name = format!("{} - Copy", cloned.index.name);
            cloned.index.settings_file = uuid::Uuid::new_v4().to_string();
            cloned.index.file_name = uuid::Uuid::new_v4().to_string();
            self.config.profiles.push(cloned);
            self.config.active_profile = self.config.profiles.len().saturating_sub(1);
            self.set_status(&t!("status.profile_copied"));
        }
    }

    fn delete_profile(&mut self) {
        if self.config.profiles.len() <= 1 {
            self.set_status(&t!("status.profile_keep_one"));
            return;
        }
        let idx = self.config.active_profile;
        let profile = &self.config.profiles[idx];
        // 删除文件
        let _ = crate::config::delete_profile(profile);
        self.config.profiles.remove(idx);
        self.config.active_profile = self.config.profiles.len().saturating_sub(1);
        self.set_status(&t!("status.profile_deleted"));
    }

    pub fn set_status(&mut self, msg: &str) {
        // 已废弃，使用 add_log 代替
        self.add_log(LogEntryType::Info, msg, None);
    }
    
    /// 添加日志条目
    pub fn add_log(&mut self, entry_type: LogEntryType, message: &str, action: Option<LogAction>) {
        self.logs.push(LogEntry {
            timestamp: Instant::now(),
            entry_type,
            message: message.to_string(),
            action,
        });
        
        // 限制日志数量，保留最近 50 条
        if self.logs.len() > 50 {
            self.logs.remove(0);
        }
    }
    
    /// 显示日志区域
    fn show_log_area(&mut self, ui: &mut egui::Ui) {
        // 限制日志区域宽度为可用宽度的 70%
        let max_width = ui.available_width() * 0.7;
        
        ui.vertical(|ui| {
            ui.set_max_width(max_width);
            ui.set_min_height(200.0);
            ui.set_max_height(300.0);
            
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_max_width(max_width);
                    
                    if self.logs.is_empty() {
                        ui.label(
                            RichText::new(t!("log.ready"))
                                .size(12.0)
                                .color(egui::Color32::from_rgb(150, 150, 150))
                        );
                    } else {
                        let logs = self.logs.clone();
                        for log in &logs {
                            self.show_log_entry(ui, log);
                        }
                    }
                });
        });
    }
    
    /// 显示单个日志条目
    fn show_log_entry(&mut self, ui: &mut egui::Ui, log: &LogEntry) {
        ui.horizontal_wrapped(|ui| {
            // 图标和颜色
            let (icon, color) = match log.entry_type {
                LogEntryType::Info => ("ℹ", egui::Color32::from_rgb(150, 150, 200)),
                LogEntryType::Success => ("✓", egui::Color32::from_rgb(100, 200, 100)),
                LogEntryType::Warning => ("⚠", egui::Color32::from_rgb(200, 200, 100)),
                LogEntryType::Error => ("✗", egui::Color32::from_rgb(200, 100, 100)),
                LogEntryType::Checking => ("⟳", egui::Color32::from_rgb(150, 150, 200)),
            };
            
            ui.label(RichText::new(icon).size(14.0).color(color));
            
            // 使用 wrap 模式显示文本，自动换行
            ui.label(
                RichText::new(&log.message)
                    .size(12.0)
                    .color(egui::Color32::from_rgb(200, 200, 200))
            );
            
            // 显示操作按钮
            if let Some(action) = &log.action {
                match action {
                    LogAction::UpdateLauncher => {
                        if !self.downloading_launcher && !self.launcher_restarting {
                            let btn = egui::Button::new("🔄 更新")
                                .fill(egui::Color32::from_rgb(80, 120, 200))
                                .min_size(egui::vec2(60.0, 20.0));
                            if ui.add(btn).clicked() {
                                self.start_launcher_update();
                            }
                        }
                    }
                    LogAction::UpdateOpenUO => {
                        if self.download_rx.is_none() {
                            let btn = egui::Button::new("🔄 更新")
                                .fill(egui::Color32::from_rgb(80, 120, 200))
                                .min_size(egui::vec2(60.0, 20.0));
                            if ui.add(btn).clicked() {
                                self.start_download();
                            }
                        }
                    }
                    LogAction::RetryDownload => {
                        if self.download_rx.is_none() {
                            let btn = egui::Button::new("🔄 重试")
                                .fill(egui::Color32::from_rgb(200, 120, 80))
                                .min_size(egui::vec2(60.0, 20.0));
                            if ui.add(btn).clicked() {
                                self.download_failed = false;
                                if self.downloading_launcher {
                                    self.start_launcher_update();
                                } else {
                                    self.start_download();
                                }
                            }
                        }
                    }
                }
            }
        });
        
        // 显示下载进度条
        if let Some((cur, total)) = self.download_progress {
            if total > 0 {
                let progress = (cur as f32) / (total as f32);
                let total_mb = (total as f32) / (1024.0 * 1024.0);
                let cur_mb = (cur as f32) / (1024.0 * 1024.0);
                
                ui.add(
                    egui::ProgressBar::new(progress)
                        .text(format!("{:.1}/{:.1} MB", cur_mb, total_mb))
                        .desired_width(ui.available_width() - 30.0)
                );
            }
        }
        
        ui.add_space(4.0);
    }

    pub fn set_screen_info(&mut self, width: u32, height: u32, scale_factor: f64) {
        self.screen_info = Some(ScreenInfo {
            width,
            height,
            scale_factor,
            is_hidpi: scale_factor > 1.0,
            lang: crate::i18n::current_locale(),
            os: crate::system_info::os_name(),
        });
    }

    fn save_config_with_screen_info(&mut self) -> Result<()> {
        // 保存所有档案，带上屏幕信息
        for profile in &self.config.profiles {
            save_profile_with_screen_info(profile, self.screen_info.clone())?;
        }
        Ok(())
    }
}

fn poll_download_channel(
    download_rx: &mut Option<mpsc::Receiver<DownloadEvent>>,
    download_progress: &mut Option<(u64, u64)>,
    downloading_launcher: &mut bool,
    launcher_restarting: &mut bool,
    status: &mut String,
    open_uo_version: &mut Option<String>,
) {
    if let Some(rx) = download_rx {
        let events: Vec<_> = rx.try_iter().collect();
        for event in events {
            match event {
                DownloadEvent::Progress { received, total } => {
                    *download_progress = Some((received, total));
                }
                DownloadEvent::Finished(result) => {
                    *download_rx = None;
                    *download_progress = None;
                    *downloading_launcher = false; // 重置下载标记
                    match result {
                        Ok(tag) => {
                            // 判断是否是 Launcher 更新并需要重启
                            if tag.starts_with("UPDATE_AND_RESTART:") {
                                // Launcher 更新完成，程序即将退出
                                let version = tag.strip_prefix("UPDATE_AND_RESTART:").unwrap_or("");
                                *status = t!("status.launcher_update_complete", version = version).to_string();
                                *launcher_restarting = true; // 标记正在重启
                                // 延迟退出，让用户看到消息
                                std::thread::spawn(|| {
                                    std::thread::sleep(std::time::Duration::from_secs(2));
                                    std::process::exit(0);
                                });
                            } else {
                                // OpenUO 下载完成
                                *open_uo_version = Some(tag.clone());
                                *status = t!("status.download_complete", version = &tag).to_string();
                            }
                        }
                        Err(_err) => {
                            *status = t!("status.download_failed").to_string();
                        }
                    }
                }
            }
        }
    }
}

fn poll_update_channel(
    update_rx: &mut Option<mpsc::Receiver<UpdateEvent>>,
    remote_open_uo: &mut Option<String>,
    remote_launcher: &mut Option<String>,
    status: &mut String,
    checking_open_uo: &mut bool,
    checking_launcher: &mut bool,
) {
    if let Some(rx) = update_rx {
        let events: Vec<_> = rx.try_iter().collect();
        for event in events {
            match event {
                UpdateEvent::OpenUO(res) => {
                    *checking_open_uo = false;
                    match res {
                        Ok(v) => *remote_open_uo = Some(v),
                        Err(_e) => {
                            *remote_open_uo = None;
                            *status = t!("status.openuo_check_failed").to_string();
                        }
                    }
                }
                UpdateEvent::Launcher(res) => {
                    *checking_launcher = false;
                    match res {
                        Ok(v) => *remote_launcher = Some(v),
                        Err(_e) => {
                            *remote_launcher = None;
                            *status = t!("status.launcher_check_failed").to_string();
                        }
                    }
                }
                UpdateEvent::Done => {}
            }
        }
    }
}

fn load_embedded_texture(
    ctx: &egui::Context,
    name: &str,
    bytes: &[u8],
) -> Option<egui::TextureHandle> {
    if let Ok(img) = image::load_from_memory(bytes) {
        let mut rgba = img.to_rgba8();

        // For logo images, make dark pixels transparent
        if name.contains("logo") {
            for pixel in rgba.chunks_exact_mut(4) {
                let r = pixel[0] as f32;
                let g = pixel[1] as f32;
                let b = pixel[2] as f32;
                let brightness = (r + g + b) / 3.0;

                // Make dark pixels (black background) transparent
                if brightness < 30.0 {
                    pixel[3] = 0;
                }
            }
        }

        let size = [img.width() as usize, img.height() as usize];
        let color_image = ColorImage::from_rgba_unmultiplied(size, &rgba);
        Some(ctx.load_texture(name, color_image, egui::TextureOptions::LINEAR))
    } else {
        None
    }
}

fn paint_background(
    ui: &egui::Ui,
    background: &Option<egui::TextureHandle>,
    logo: &Option<egui::TextureHandle>,
) {
    let rect = ui.max_rect();
    let painter = ui.painter();

    if let Some(bg) = background {
        let tex_size = bg.size_vec2();
        let avail = rect.size();
        let scale = (avail.x / tex_size.x).max(avail.y / tex_size.y);
        let size = tex_size * scale;
        let offset = (avail - size) * 0.5;
        let dest = egui::Rect::from_min_size(rect.min + offset, size);
        painter.image(
            bg.id(),
            dest,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    }

    if let Some(logo) = logo {
        let mut size = logo.size_vec2();
        let max_width = 180.0;
        if size.x > max_width {
            let ratio = max_width / size.x;
            size.x = max_width;
            size.y *= ratio;
        }
        let margin = 12.0;
        let padding = 8.0;
        let pos = egui::pos2(
            rect.max.x - size.x - margin - padding * 2.0,
            rect.min.y + margin,
        );
        let logo_rect =
            egui::Rect::from_min_size(egui::pos2(pos.x + padding, pos.y + padding), size);
        let bg_rect = egui::Rect::from_min_size(
            pos,
            egui::vec2(size.x + padding * 2.0, size.y + padding * 2.0),
        );

        // Draw shadow
        let shadow_offset = 4.0;
        let shadow_rect = bg_rect.translate(egui::vec2(shadow_offset, shadow_offset));
        painter.rect(
            shadow_rect,
            8.0,
            Color32::from_rgba_unmultiplied(0, 0, 0, 60),
            egui::Stroke::NONE,
        );

        // Draw semi-transparent dark background
        painter.rect(
            bg_rect,
            8.0,
            Color32::from_rgba_unmultiplied(0, 0, 0, 77),
            egui::Stroke::NONE,
        );

        // Draw logo
        painter.image(
            logo.id(),
            logo_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    }
}
