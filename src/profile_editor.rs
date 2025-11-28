use crate::config::ProfileConfig;
use crate::crypter;
use crate::i18n::t;

fn pick_directory(current: &str) -> Option<String> {
    let mut dialog = rfd::FileDialog::new();
    if !current.is_empty() {
        dialog = dialog.set_directory(current);
    }
    dialog
        .pick_folder()
        .map(|p| p.to_string_lossy().to_string())
}

pub struct ProfileEditor {
    pub editor_profile: Option<ProfileConfig>,
    pub editor_index: Option<usize>,
}

impl ProfileEditor {
    pub fn new() -> Self {
        Self {
            editor_profile: None,
            editor_index: None,
        }
    }

    pub fn open(&mut self, mut profile: ProfileConfig, index: usize) {
        // 解密密码用于显示
        profile.settings.password = crypter::decrypt(&profile.settings.password);
        
        // 如果 UO 资源目录为空，默认设置为启动器所在目录
        if profile.settings.ultima_online_directory.is_empty() {
            let launcher_dir = crate::config::base_dir();
            profile.settings.ultima_online_directory = launcher_dir.to_string_lossy().to_string();
        }
        
        self.editor_index = Some(index);
        self.editor_profile = Some(profile);
    }

    pub fn close(&mut self) {
        self.editor_profile = None;
        self.editor_index = None;
    }

    pub fn is_open(&self) -> bool {
        self.editor_profile.is_some()
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Option<(usize, ProfileConfig)> {
        if self.editor_profile.is_none() {
            return None;
        }

        let mut open = true;
        let mut result = None;

        egui::Window::new(t!("profile_editor.title"))
            .open(&mut open)
            .frame(egui::Frame::window(&ctx.style()).fill(egui::Color32::from_rgb(40, 40, 45)))
            .show(ctx, |ui| {
                if let Some(profile) = self.editor_profile.as_mut() {
                    ui.horizontal(|ui| {
                        ui.label(t!("profile_editor.name"));
                        ui.text_edit_singleline(&mut profile.index.name);
                    });

                    ui.separator();
                    ui.label(t!("profile_editor.server_settings"));

                    ui.horizontal(|ui| {
                        ui.label(t!("profile_editor.server_host"));
                        ui.text_edit_singleline(&mut profile.settings.ip);
                    });
                    ui.horizontal(|ui| {
                        ui.label(t!("profile_editor.server_port"));
                        ui.add(egui::DragValue::new(&mut profile.settings.port).speed(1));
                    });

                    ui.separator();
                    ui.label(t!("profile_editor.account_settings"));

                    ui.horizontal(|ui| {
                        ui.label(t!("profile_editor.username"));
                        ui.text_edit_singleline(&mut profile.settings.username);
                    });
                    ui.horizontal(|ui| {
                        ui.label(t!("profile_editor.password"));
                        ui.add(
                            egui::TextEdit::singleline(&mut profile.settings.password)
                                .password(true),
                        );
                    });
                    ui.checkbox(&mut profile.settings.save_account, t!("profile_editor.save_account").as_ref());

                    ui.separator();
                    ui.label(t!("profile_editor.game_settings"));

                    ui.horizontal(|ui| {
                        ui.label(t!("profile_editor.uo_directory"));
                        ui.text_edit_singleline(&mut profile.settings.ultima_online_directory);
                        let browse_btn = egui::Button::new(t!("profile_editor.browse"))
                            .fill(egui::Color32::from_rgb(100, 100, 120))
                            .min_size(egui::vec2(60.0, 20.0));
                        if ui.add(browse_btn).clicked() {
                            if let Some(path) = pick_directory(&profile.settings.ultima_online_directory) {
                                profile.settings.ultima_online_directory = path;
                            }
                        }
                    });
                    
                    // 显示当前 UO 版本号和加密设置
                    if !profile.settings.ultima_online_directory.is_empty() {
                        let client_exe = std::path::Path::new(&profile.settings.ultima_online_directory).join("client.exe");
                        if client_exe.exists() {
                            if let Some(version) = crate::version_reader::read_pe_version(&client_exe) {
                                // 显示版本号
                                ui.label(egui::RichText::new(format!("{}: {}", t!("profile_editor.client_version"), version)).size(11.0).color(egui::Color32::from_rgb(150, 150, 150)));
                                
                                // 自动更新 client_version 字段
                                if profile.settings.client_version != version {
                                    profile.settings.client_version = version.clone();
                                }
                                
                                // 根据版本号推荐加密类型（如果没有强制禁用加密）
                                if !profile.settings.force_no_encryption {
                                    let suggested = crate::encryption_helper::suggest_encryption_from_version(&version);
                                    if profile.settings.encryption != suggested {
                                        profile.settings.encryption = suggested;
                                    }
                                }
                                
                                // 显示当前加密状态
                                let encryption_text = if profile.settings.force_no_encryption {
                                    t!("profile_editor.encryption_disabled")
                                } else if profile.settings.encryption == 1 {
                                    t!("profile_editor.encryption_enabled")
                                } else {
                                    t!("profile_editor.encryption_none")
                                };
                                ui.label(egui::RichText::new(format!("{}: {}", t!("profile_editor.encryption_status"), encryption_text)).size(11.0).color(egui::Color32::from_rgb(150, 150, 150)));
                            } else {
                                ui.label(egui::RichText::new(t!("profile_editor.client_found")).size(11.0).color(egui::Color32::from_rgb(100, 200, 100)));
                            }
                        } else {
                            ui.label(egui::RichText::new(t!("profile_editor.client_not_found")).size(11.0).color(egui::Color32::from_rgb(200, 100, 100)));
                        }
                    }
                    
                    // 强制禁用加密的选项
                    ui.checkbox(&mut profile.settings.force_no_encryption, t!("profile_editor.force_no_encryption").as_ref());

                    ui.horizontal(|ui| {
                        ui.label(t!("profile_editor.last_character"));
                        ui.text_edit_singleline(&mut profile.index.last_character_name);
                    });
                    
                    // 自动登录和掉线重连排在一行
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut profile.settings.auto_login, t!("profile_editor.auto_login").as_ref());
                        ui.checkbox(&mut profile.settings.reconnect, t!("profile_editor.reconnect").as_ref());
                    });
                    ui.horizontal(|ui| {
                        ui.label(t!("profile_editor.additional_args"));
                        ui.text_edit_singleline(&mut profile.index.additional_args);
                    });
                }

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    let save_btn = egui::Button::new(
                        egui::RichText::new(t!("profile_editor.save")).size(14.0)
                    )
                    .fill(egui::Color32::from_rgb(50, 120, 200))
                    .min_size(egui::vec2(80.0, 32.0));
                    
                    if ui.add(save_btn).clicked() {
                        if let (Some(idx), Some(profile)) =
                            (self.editor_index, self.editor_profile.clone())
                        {
                            result = Some((idx, profile));
                        }
                        self.close();
                    }
                    
                    let cancel_btn = egui::Button::new(
                        egui::RichText::new(t!("profile_editor.cancel")).size(14.0)
                    )
                    .fill(egui::Color32::from_rgb(80, 80, 90))
                    .min_size(egui::vec2(80.0, 32.0));
                    
                    if ui.add(cancel_btn).clicked() {
                        self.close();
                    }
                });
            });

        if !open {
            self.close();
        }

        result
    }
}
