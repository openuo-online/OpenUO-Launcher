// 在 Windows 上隐藏控制台窗口（GUI 应用）
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// 初始化 i18n（必须在最前面）
rust_i18n::i18n!("locales", fallback = "en");

mod config;
mod crypter;
mod encryption_helper;
mod github;
mod i18n;
mod profile_editor;
mod system_info;
mod ui;
mod version_reader;

use anyhow::{Context, Result};
use egui_wgpu::{Renderer, ScreenDescriptor};
use egui_winit::{pixels_per_point, State as EguiWinitState};
use std::sync::Arc;
use tracing::info;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

use config::load_config_from_disk;
use ui::LauncherUi;

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();
}

fn get_primary_screen_size() -> (u32, u32) {
    #[cfg(target_os = "macos")]
    {
        use core_graphics::display::CGDisplay;
        let main_display = CGDisplay::main();
        let mode = main_display.display_mode().unwrap();
        (mode.width() as u32, mode.height() as u32)
    }
    
    #[cfg(target_os = "windows")]
    {
        unsafe {
            use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
            let width = GetSystemMetrics(SM_CXSCREEN) as u32;
            let height = GetSystemMetrics(SM_CYSCREEN) as u32;
            (width, height)
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        // Linux 下使用默认值
        // 注意：实际分辨率会通过 winit 的 window.scale_factor() 正确处理
        // 如需精确获取，可以使用 X11 (libxrandr) 或 Wayland API
        // 但由于 winit 已经处理了 DPI 缩放，这个默认值通常足够
        (1920, 1080)
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        (1920, 1080)
    }
}

fn main() -> Result<()> {
    init_tracing();
    
    // 加载保存的语言设置
    let launcher_settings = config::load_launcher_settings();
    
    // 初始化国际化（优先使用保存的语言）
    i18n::init_locale_with_saved(launcher_settings.language);
    
    pollster::block_on(run())
}

async fn run() -> Result<()> {
    let event_loop = EventLoop::new().context("Failed to create event loop")?;
    
    // 加载窗口图标
    let window_icon = load_window_icon();
    
    let mut window_builder = WindowBuilder::new()
        .with_title("OpenUO Launcher")
        .with_inner_size(LogicalSize::new(960.0, 600.0))
        .with_min_inner_size(LogicalSize::new(720.0, 480.0));
    
    // 设置窗口图标（如果加载成功）
    if let Some(icon) = window_icon {
        window_builder = window_builder.with_window_icon(Some(icon));
    }
    
    let window = Arc::new(
        window_builder
            .build(&event_loop)
            .context("Failed to create window")?,
    );

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        dx12_shader_compiler: Default::default(),
        // 禁用验证层以避免 DirectX 12 的资源状态警告
        flags: wgpu::InstanceFlags::empty(),
        gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
    });
    let surface = instance.create_surface(window.clone()).context("surface")?;
    
    let mut adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await;
    if adapter.is_none() {
        adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await;
    }
    if adapter.is_none() {
        adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: true,
                compatible_surface: Some(&surface),
            })
            .await;
    }
    if adapter.is_none() {
        adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: true,
                compatible_surface: Some(&surface),
            })
            .await;
    }

    let adapter = adapter.context("No compatible GPU adapter found")?;

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("wgpu-device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        )
        .await?;

    let caps = surface.get_capabilities(&adapter);
    let surface_format = caps
        .formats
        .iter()
        .copied()
        .find(|format| format.is_srgb())
        .unwrap_or(caps.formats[0]);

    let size = window.inner_size();
    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width.max(1),
        height: size.height.max(1),
        present_mode: caps.present_modes[0],
        alpha_mode: caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &config);

    let egui_ctx = egui::Context::default();
    install_cjk_font(&egui_ctx);
    let mut egui_state = EguiWinitState::new(
        egui_ctx.clone(),
        egui::ViewportId::ROOT,
        &window,
        Some(window.scale_factor() as f32),
        None,
    );
    let mut egui_renderer = Renderer::new(&device, surface_format, None, 1);

    let loaded_config = load_config_from_disk();
    let mut ui = LauncherUi::new(loaded_config);

    // 获取屏幕信息
    let scale_factor = window.scale_factor();
    let (screen_width, screen_height) = get_primary_screen_size();
    
    ui.set_screen_info(screen_width, screen_height, scale_factor);
    info!(
        "{}: {}x{} @ {:.2}x scale (HiDPI: {})",
        i18n::t!("log.screen_info"),
        screen_width,
        screen_height,
        scale_factor,
        scale_factor > 1.0
    );

    info!("{}", i18n::t!("log.launcher_initialized"));

    event_loop.run(move |event, target| match event {
        Event::WindowEvent { event, window_id } if window_id == window.id() => {
            let response = egui_state.on_window_event(&window, &event);
            if response.consumed {
                return;
            }

            match event {
                WindowEvent::CloseRequested => target.exit(),
                WindowEvent::Resized(new_size) => {
                    if new_size.width > 0 && new_size.height > 0 {
                        config.width = new_size.width;
                        config.height = new_size.height;
                        surface.configure(&device, &config);
                        window.request_redraw();
                    }
                }
                WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                    let new_size = window.inner_size();
                    egui_ctx.set_pixels_per_point(scale_factor as f32);
                    config.width = new_size.width.max(1);
                    config.height = new_size.height.max(1);
                    surface.configure(&device, &config);
                    window.request_redraw();
                }
                WindowEvent::RedrawRequested => {
                    let input = egui_state.take_egui_input(&window);

                    let full_output = egui_ctx.run(input, |ctx| {
                        ctx.request_repaint();
                        ui.ui(ctx);
                    });

                    egui_state.handle_platform_output(&window, full_output.platform_output);

                    let screen_descriptor = ScreenDescriptor {
                        size_in_pixels: [config.width, config.height],
                        pixels_per_point: pixels_per_point(&egui_ctx, &window),
                    };

                    let paint_jobs =
                        egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);

                    for (id, image_delta) in &full_output.textures_delta.set {
                        egui_renderer.update_texture(&device, &queue, *id, image_delta);
                    }

                    let surface_tex = match surface.get_current_texture() {
                        Ok(frame) => frame,
                        Err(wgpu::SurfaceError::Lost) => {
                            surface.configure(&device, &config);
                            return;
                        }
                        Err(wgpu::SurfaceError::Outdated) => {
                            return;
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            target.exit();
                            return;
                        }
                        Err(wgpu::SurfaceError::Timeout) => {
                            return;
                        }
                    };

                    let view = surface_tex
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    let mut encoder =
                        device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("egui encoder"),
                        });

                    egui_renderer.update_buffers(
                        &device,
                        &queue,
                        &mut encoder,
                        &paint_jobs,
                        &screen_descriptor,
                    );

                    {
                        let mut render_pass =
                            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("egui render pass"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: &view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(wgpu::Color {
                                            r: 0.1,
                                            g: 0.1,
                                            b: 0.1,
                                            a: 1.0,
                                        }),
                                        store: wgpu::StoreOp::Store,
                                    },
                                })],
                                depth_stencil_attachment: None,
                                timestamp_writes: None,
                                occlusion_query_set: None,
                            });

                        egui_renderer.render(&mut render_pass, &paint_jobs, &screen_descriptor);
                    }

                    queue.submit(std::iter::once(encoder.finish()));
                    surface_tex.present();

                    for id in &full_output.textures_delta.free {
                        egui_renderer.free_texture(id);
                    }

                    if full_output.viewport_output[&egui::ViewportId::ROOT]
                        .repaint_delay
                        .is_zero()
                    {
                        window.request_redraw();
                    }
                }
                _ => {}
            }
        }
        Event::AboutToWait => {
            window.request_redraw();
        }
        _ => {}
    })?;

    Ok(())
}

fn load_window_icon() -> Option<winit::window::Icon> {
    // 辅助函数：尝试从字节加载图标
    let load_icon_from_bytes = |bytes: &[u8]| -> Option<winit::window::Icon> {
        let img = image::load_from_memory(bytes).ok()?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        winit::window::Icon::from_rgba(rgba.into_raw(), width, height).ok()
    };

    // 1. Windows 上优先尝试 icon.ico
    #[cfg(target_os = "windows")]
    {
        if let Some(icon) = load_icon_from_bytes(include_bytes!("../assets/icon.ico")) {
            tracing::info!("{}", i18n::t!("log.icon_loaded"));
            return Some(icon);
        }
    }

    // 2. 其他情况（或 Windows 加载 ico 失败）使用 logo.png
    if let Some(icon) = load_icon_from_bytes(include_bytes!("../assets/logo.png")) {
        tracing::info!("{}", i18n::t!("log.icon_loaded"));
        return Some(icon);
    }

    tracing::warn!("{}", i18n::t!("log.icon_create_failed"));
    None
}

fn install_cjk_font(ctx: &egui::Context) {
    use std::fs;
    let mut fonts = egui::FontDefinitions::default();
    
    // Windows 字体路径
    #[cfg(target_os = "windows")]
    let candidates = [
        "C:\\Windows\\Fonts\\msyh.ttc",      // 微软雅黑
        "C:\\Windows\\Fonts\\msyhbd.ttc",    // 微软雅黑 Bold
        "C:\\Windows\\Fonts\\simhei.ttf",    // 黑体
        "C:\\Windows\\Fonts\\simsun.ttc",    // 宋体
        "C:\\Windows\\Fonts\\simkai.ttf",    // 楷体
    ];
    
    // macOS 字体路径
    #[cfg(target_os = "macos")]
    let candidates = [
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/Hiragino Sans GB W3.ttc",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
    ];
    
    // Linux 字体路径
    #[cfg(target_os = "linux")]
    let candidates = [
        // Noto CJK (最常见)
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansSC-Regular.otf",
        // WenQuanYi (文泉驿)
        "/usr/share/fonts/wenquanyi/wqy-microhei/wqy-microhei.ttc",
        "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
        // Droid Sans Fallback
        "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
        // AR PL UMing (文鼎)
        "/usr/share/fonts/truetype/arphic/uming.ttc",
    ];

    let font_id = "cjk-fallback";
    let loaded = candidates
        .iter()
        .find_map(|path| fs::read(path).ok().map(|bytes| (path, bytes)));

    if let Some((_path, data)) = loaded {
        fonts
            .font_data
            .insert(font_id.to_string(), egui::FontData::from_owned(data));
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, font_id.to_string());
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, font_id.to_string());
        ctx.set_fonts(fonts);
    } else {
        tracing::warn!("{}", i18n::t!("log.font_not_found"));
    }
}
