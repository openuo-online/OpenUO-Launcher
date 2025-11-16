use crate::config::ProfileConfig;
use crate::crypter;

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

        egui::Window::new("编辑配置")
            .open(&mut open)
            .frame(egui::Frame::window(&ctx.style()).fill(egui::Color32::from_rgb(40, 40, 45)))
            .show(ctx, |ui| {
                if let Some(profile) = self.editor_profile.as_mut() {
                    ui.horizontal(|ui| {
                        ui.label("配置名称:");
                        ui.text_edit_singleline(&mut profile.index.name);
                    });

                    ui.separator();
                    ui.label("服务器设置");

                    ui.horizontal(|ui| {
                        ui.label("服务器地址:");
                        ui.text_edit_singleline(&mut profile.settings.ip);
                    });
                    ui.horizontal(|ui| {
                        ui.label("端口:");
                        ui.add(egui::DragValue::new(&mut profile.settings.port).speed(1));
                    });

                    ui.separator();
                    ui.label("账号设置");

                    ui.horizontal(|ui| {
                        ui.label("账号:");
                        ui.text_edit_singleline(&mut profile.settings.username);
                    });
                    ui.horizontal(|ui| {
                        ui.label("密码:");
                        ui.add(
                            egui::TextEdit::singleline(&mut profile.settings.password)
                                .password(true),
                        );
                    });
                    ui.checkbox(&mut profile.settings.save_account, "保存账号密码");

                    ui.separator();
                    ui.label("游戏设置");

                    ui.label(egui::RichText::new("⚠ UO 资源目录（包含 client.exe 的目录）").size(12.0).color(egui::Color32::from_rgb(200, 200, 100)));
                    ui.horizontal(|ui| {
                        ui.label("UO 资源目录:");
                        ui.text_edit_singleline(&mut profile.settings.ultima_online_directory);
                        let browse_btn = egui::Button::new("📁 浏览")
                            .fill(egui::Color32::from_rgb(100, 100, 120))
                            .min_size(egui::vec2(60.0, 20.0));
                        if ui.add(browse_btn).clicked() {
                            if let Some(path) = pick_directory(&profile.settings.ultima_online_directory) {
                                profile.settings.ultima_online_directory = path;
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("角色名:");
                        ui.text_edit_singleline(&mut profile.index.last_character_name);
                    });
                    ui.checkbox(&mut profile.settings.auto_login, "自动登录");
                    ui.checkbox(&mut profile.settings.reconnect, "掉线重连");
                    ui.horizontal(|ui| {
                        ui.label("附加参数:");
                        ui.text_edit_singleline(&mut profile.index.additional_args);
                    });
                }

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    let save_btn = egui::Button::new(
                        egui::RichText::new("💾 保存").size(14.0)
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
                        egui::RichText::new("✖ 取消").size(14.0)
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
