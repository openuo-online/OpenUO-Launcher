mod config;
mod github;
mod profile_editor;
mod ui;

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

fn main() -> Result<()> {
    init_tracing();
    pollster::block_on(run())
}

async fn run() -> Result<()> {
    let event_loop = EventLoop::new().context("Failed to create event loop")?;
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Another OpenUO Launcher")
            .with_inner_size(LogicalSize::new(960.0, 600.0))
            .with_min_inner_size(LogicalSize::new(720.0, 480.0))
            .build(&event_loop)
            .context("Failed to create window")?,
    );

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        dx12_shader_compiler: Default::default(),
        flags: wgpu::InstanceFlags::default(),
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

    info!("Launcher initialized");

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

fn install_cjk_font(ctx: &egui::Context) {
    use std::fs;
    let mut fonts = egui::FontDefinitions::default();
    let candidates = [
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/Hiragino Sans GB W3.ttc",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansSC-Regular.otf",
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
    }
}
