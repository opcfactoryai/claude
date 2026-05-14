use eframe::egui::{self, Align, Color32, RichText, Rounding, Vec2};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::auth::LoginState;
use crate::launcher::show_launch_error;
use crate::oauth::login;

// ── 配色 ─────────────────────────────────────────────────────

const ACCENT: Color32 = Color32::from_rgb(79, 195, 247);
const ACCENT_GREEN: Color32 = Color32::from_rgb(102, 187, 106);
const ACCENT_RED: Color32 = Color32::from_rgb(239, 83, 80);

const TEXT_PRIMARY: Color32 = Color32::from_rgb(224, 224, 224);
const TEXT_SECONDARY: Color32 = Color32::from_rgb(158, 158, 158);
const TEXT_HINT: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 80);
const DISABLED_BG: Color32 = Color32::from_rgb(55, 55, 60);
const DISABLED_TEXT: Color32 = Color32::from_rgb(120, 120, 125);

// ── 按钮样式（由状态派生，消除 if/else）───────────────────

struct BtnStyle {
    text: String,
    text_color: Color32,
    fill: Color32,
    enabled: bool,
}

impl LoginState {
    fn login_btn(&self) -> BtnStyle {
        match self {
            LoginState::LoggedOut => BtnStyle {
                text: "登  录".into(),
                text_color: Color32::WHITE,
                fill: ACCENT,
                enabled: true,
            },
            LoginState::LoggingIn(_) => BtnStyle {
                text: "登录中...".into(),
                text_color: DISABLED_TEXT,
                fill: DISABLED_BG,
                enabled: false,
            },
            LoginState::LoggedIn(_) => BtnStyle {
                text: "已登录".into(),
                text_color: DISABLED_TEXT,
                fill: DISABLED_BG,
                enabled: false,
            },
            LoginState::Launching => BtnStyle {
                text: "登  录".into(),
                text_color: DISABLED_TEXT,
                fill: DISABLED_BG,
                enabled: false,
            },
        }
    }

    fn launch_btn(&self) -> BtnStyle {
        let ok = matches!(self, LoginState::LoggedIn(_));
        BtnStyle {
            text: "启  动".into(),
            text_color: if ok { Color32::WHITE } else { DISABLED_TEXT },
            fill: if ok { ACCENT } else { DISABLED_BG },
            enabled: ok,
        }
    }
}

// ── App ──────────────────────────────────────────────────────

pub struct DesignGPTApp {
    root: PathBuf,
    state: LoginState,
    window_shown: bool,
    error_msg: Option<String>,
}

impl DesignGPTApp {
    pub fn new(root: PathBuf) -> Self {
        // 启动时唯一入口：auth_check 校验（auth.json 结构 + JWT 过期）
        if let Some(user) = crate::auth_check::check_login(&root) {
            return Self { root, state: LoginState::LoggedIn(user), window_shown: false, error_msg: None };
        }

        Self { root, state: LoginState::LoggedOut, window_shown: false, error_msg: None }
    }
}

impl eframe::App for DesignGPTApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());

        if !self.window_shown {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            self.window_shown = true;
        }

        self.poll_login(ctx);

        if matches!(self.state, LoginState::Launching) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            let root = self.root.clone();
            std::thread::spawn(move || match crate::launcher::launch_claude(&root) {
                Ok(code) => std::process::exit(code),
                Err(e) => {
                    show_launch_error(&e.to_string());
                    std::process::exit(1);
                }
            });
            return;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_content(ui);
        });
    }
}

impl DesignGPTApp {
    fn poll_login(&mut self, ctx: &egui::Context) {
        let transition = match &self.state {
            LoginState::LoggingIn(pending) => {
                if let Some(result) = pending.lock().unwrap().take() {
                    Some(result)
                } else {
                    ctx.request_repaint_after(Duration::from_millis(100));
                    None
                }
            }
            _ => None,
        };

        if let Some(result) = transition {
            match result {
                Ok(user) => {
                    self.error_msg = None;
                    self.state = LoginState::LoggedIn(user);
                }
                Err(e) => {
                    self.error_msg = Some(e);
                    self.state = LoginState::LoggedOut;
                }
            }
        }
    }

    // ── UI 渲染 ──────────────────────────────────────────

    fn render_content(&mut self, ui: &mut egui::Ui) {
        let available = ui.available_size();
        let max_w = 400.0_f32.min(available.x - 40.0);

        ui.vertical_centered(|ui| {
            ui.set_max_width(max_w);
            ui.add_space(40.0);

            ui.label(RichText::new("Claude").size(28.0).color(ACCENT));
            ui.add_space(24.0);

            self.render_account_card(ui);
            ui.add_space(20.0);

            if let Some(ref err) = self.error_msg {
                egui::Frame::none()
                    .fill(Color32::from_rgba_premultiplied(239, 83, 80, 40))
                    .rounding(Rounding::same(6.0))
                    .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(RichText::new(err.as_str()).size(12.0).color(ACCENT_RED));
                    });
                ui.add_space(12.0);
            }

            self.render_buttons(ui);
        });

        ui.with_layout(egui::Layout::bottom_up(Align::Center), |ui| {
            ui.add_space(8.0);
            ui.label(RichText::new("武汉市向量求索信息技术有限公司").size(11.0).color(TEXT_HINT));
        });
    }

    fn render_account_card(&self, ui: &mut egui::Ui) {
        match &self.state {
            LoginState::LoggedOut => {
                ui.add_space(8.0);
                ui.label(RichText::new("请点击下方登录按钮跳转至官网完成授权").size(13.0).color(TEXT_SECONDARY));
                ui.add_space(8.0);
            }
            LoginState::LoggingIn(_) => {
                ui.add_space(8.0);
                ui.label(RichText::new("请在浏览器中完成授权，完成后即可启动").size(13.0).color(ACCENT_GREEN));
                ui.add_space(8.0);
            }
            LoginState::LoggedIn(user) => {
                ui.label(RichText::new("账户信息").size(12.0).color(TEXT_HINT));
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(12.0);

                ui.horizontal(|ui| {
                    ui.label(RichText::new("手机号").size(12.0).color(TEXT_HINT));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(&user.phone).size(14.0).color(TEXT_PRIMARY));
                    });
                });

                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.label(RichText::new("余额").size(12.0).color(TEXT_HINT));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(&user.balance).size(16.0).color(ACCENT));
                    });
                });

                ui.add_space(10.0);
                ui.separator();
            }
            LoginState::Launching => {}
        }
    }

    // ── 按钮 ──────────────────────────────────────────────

    fn render_buttons(&mut self, ui: &mut egui::Ui) {
        let login_cfg = self.state.login_btn();
        let login_clicked = render_btn(ui, &login_cfg)
            && login_cfg.enabled
            && matches!(self.state, LoginState::LoggedOut);
        if login_clicked {
            self.error_msg = None;
            self.start_login();
        }

        ui.add_space(10.0);

        let launch_cfg = self.state.launch_btn();
        let launch_clicked = render_btn(ui, &launch_cfg)
            && launch_cfg.enabled
            && matches!(self.state, LoginState::LoggedIn(_));
        if launch_clicked {
            self.state = LoginState::Launching;
        }
    }

    fn start_login(&mut self) {
        let root = self.root.clone();
        let result = Arc::new(Mutex::new(None));
        self.state = LoginState::LoggingIn(result.clone());
        std::thread::spawn(move || {
            let r = login(&root).map_err(|e| format!("登录失败：{}", e));
            *result.lock().unwrap() = Some(r);
        });
    }
}

// ── 按钮渲染（自由函数）────────────────────────────────────

fn render_btn(ui: &mut egui::Ui, cfg: &BtnStyle) -> bool {
    let btn = egui::Button::new(RichText::new(cfg.text.as_str()).size(14.0).color(cfg.text_color))
        .fill(cfg.fill)
        .min_size(Vec2::new(ui.available_width(), 40.0))
        .rounding(Rounding::same(6.0));

    if cfg.enabled {
        ui.add_sized(Vec2::new(ui.available_width(), 40.0), btn).clicked()
    } else {
        ui.add_sized(Vec2::new(ui.available_width(), 40.0), btn);
        false
    }
}
