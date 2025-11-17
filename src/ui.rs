use anyhow::{Context, Result};
use egui::{Color32, ColorImage, RichText};
use std::process::Command;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::config::*;
use crate::github::*;
use crate::i18n::t;
use crate::profile_editor::ProfileEditor;

pub struct LauncherUi {
    pub config: LauncherConfig,
    pub status: String,
    pub profile_editor: ProfileEditor,
    pub open_uo_version: Option<String>,
    pub launcher_version: String,
    pub download_rx: Option<mpsc::Receiver<DownloadEvent>>,
    pub download_progress: Option<(u64, u64)>,
    pub downloading_launcher: bool, // 标记是否正在下载 Launcher
    pub launcher_restarting: bool, // 标记 Launcher 正在重启
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
}

impl LauncherUi {
    pub fn new(config: LauncherConfig) -> Self {
        Self {
            config,
            status: format!("{}", t!("status.config_loaded").to_string()),
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
                
                // 添加左边距和上边距，与 logo 对齐
                let margin = 12.0;
                ui.add_space(margin);
                
                ui.horizontal(|ui| {
                    ui.add_space(margin);
                    ui.vertical(|ui| {
                        ui.heading(RichText::new(t!("window.title")).size(24.0).strong());
                        ui.add_space(12.0);

                        self.show_language_selector(ui);
                        ui.add_space(8.0);
                        self.show_profile_selector(ui);
                        ui.add_space(8.0);
                        self.show_version_info(ui);
                        ui.add_space(8.0);
                        self.show_launch_button(ui);
                        ui.add_space(10.0);

                        ui.label(
                            RichText::new(&self.status)
                                .italics()
                                .color(egui::Color32::LIGHT_GRAY),
                        );
                    });
                });
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
                        Ok(msg) => self.set_status(&msg),
                        Err(_err) => self.set_status(&t!("status.launch_failed")),
                    }
                }
            });
        });
    }

    fn poll_channels(&mut self) {
        poll_download_channel(
            &mut self.download_rx,
            &mut self.download_progress,
            &mut self.downloading_launcher,
            &mut self.launcher_restarting,
            &mut self.status,
            &mut self.open_uo_version,
        );
        poll_update_channel(
            &mut self.update_rx,
            &mut self.remote_open_uo,
            &mut self.remote_launcher,
            &mut self.status,
            &mut self.checking_open_uo,
            &mut self.checking_launcher,
        );
    }

    fn start_download(&mut self) {
        if self.download_rx.is_some() {
            return;
        }
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
        self.downloading_launcher = false; // 标记正在下载 OpenUO
    }

    fn start_launcher_update(&mut self) {
        if self.download_rx.is_some() {
            return;
        }
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
        self.downloading_launcher = true; // 标记正在下载 Launcher
        self.set_status(&t!("status.launcher_update_downloading"));
    }

    fn trigger_update_checks(&mut self, open_uo: bool, launcher: bool) {
        if !open_uo && !launcher {
            return;
        }
        if open_uo && !self.checking_open_uo {
            self.checking_open_uo = true;
        }
        if launcher && !self.checking_launcher {
            self.checking_launcher = true;
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
        self.status = msg.to_string();
    }

    pub fn set_screen_info(&mut self, width: u32, height: u32, scale_factor: f64) {
        self.screen_info = Some(ScreenInfo {
            width,
            height,
            scale_factor,
            is_hidpi: scale_factor > 1.0,
        });
    }

    fn save_config_with_screen_info(&mut self) -> Result<()> {
        // 保存所有档案，带上屏幕信息
        for profile in &self.config.profiles {
            save_profile_with_screen_info(profile, self.screen_info)?;
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
