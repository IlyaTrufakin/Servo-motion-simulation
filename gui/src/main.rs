#![windows_subsystem = "windows"]

use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::collections::VecDeque;

use protocol::{Command, Message, ServoParams, SimulationState};

const MAX_HISTORY: usize = 500; // Store last 500 points (~8.3 seconds at 60Hz)

struct AppState {
    params: ServoParams,
    latest_state: SimulationState,
    
    // History for plotting
    time_history: VecDeque<f64>,
    master_pos_hist: VecDeque<f64>,
    slave_pos_hist: VecDeque<f64>,
    master_vel_hist: VecDeque<f64>,
    slave_vel_hist: VecDeque<f64>,
    error_hist: VecDeque<f64>,

    // Controls
    target_vel: f64,
    target_pos: f64,
    pos_mode: bool,

    // Comms
    cmd_tx: Option<Sender<Command>>,
    msg_rx: Receiver<Message>,
    
    // Connection Status
    is_connected: bool,
    status_rx: Receiver<bool>,

    // Window Visibility Flags
    show_pid_window: bool,
    show_physics_window: bool,
    show_servo_window: bool,
    show_slave_data_window: bool,
}

impl Default for AppState {
    fn default() -> Self {
        let (_, rx) = mpsc::channel();
        let (_, status_rx) = mpsc::channel();
        Self {
            params: ServoParams::default(),
            latest_state: SimulationState::default(),
            time_history: VecDeque::with_capacity(MAX_HISTORY),
            master_pos_hist: VecDeque::with_capacity(MAX_HISTORY),
            slave_pos_hist: VecDeque::with_capacity(MAX_HISTORY),
            master_vel_hist: VecDeque::with_capacity(MAX_HISTORY),
            slave_vel_hist: VecDeque::with_capacity(MAX_HISTORY),
            error_hist: VecDeque::with_capacity(MAX_HISTORY),
            target_vel: 0.0,
            target_pos: 0.0,
            pos_mode: false,
            cmd_tx: None,
            msg_rx: rx,
            is_connected: false,
            status_rx,
            show_pid_window: false,
            show_physics_window: false,
            show_servo_window: false,
            show_slave_data_window: false,
        }
    }
}

impl AppState {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel::<Command>();
        let (msg_tx, msg_rx) = mpsc::channel::<Message>();
        let (status_tx, status_rx) = mpsc::channel::<bool>();

        // Background thread for TCP comms
        thread::spawn(move || {
            loop {
                if let Ok(mut stream) = TcpStream::connect("127.0.0.1:5000") {
                    println!("Connected to controller!");
                    let _ = status_tx.send(true);
                    let mut reader = BufReader::new(stream.try_clone().unwrap());
                    
                    // Request current params
                    let _ = stream.write_all(b"{\"Reset\":null}\n");

                    let mut line = String::new();
                    loop {
                        // Check for outgoing commands
                        while let Ok(cmd) = cmd_rx.try_recv() {
                            if let Ok(json) = serde_json::to_string(&cmd) {
                                let _ = stream.write_all(format!("{}\n", json).as_bytes());
                            }
                        }

                        // Read incoming messages
                        line.clear();
                        match reader.read_line(&mut line) {
                            Ok(0) => {
                                let _ = status_tx.send(false);
                                break; // Disconnected
                            }
                            Ok(_) => {
                                if let Ok(msg) = serde_json::from_str::<Message>(&line) {
                                    let _ = msg_tx.send(msg);
                                }
                            }
                            Err(_) => {
                                let _ = status_tx.send(false);
                                break;
                            }
                        }
                    }
                } else {
                    let _ = status_tx.send(false);
                }
                thread::sleep(std::time::Duration::from_secs(1));
            }
        });

        Self {
            cmd_tx: Some(cmd_tx),
            msg_rx,
            status_rx,
            ..Default::default()
        }
    }

    fn process_messages(&mut self) {
        while let Ok(status) = self.status_rx.try_recv() {
            self.is_connected = status;
        }

        while let Ok(msg) = self.msg_rx.try_recv() {
            match msg {
                Message::Telemetry(state) => {
                    self.latest_state = state.clone();
                    
                    if self.time_history.len() >= MAX_HISTORY {
                        self.time_history.pop_front();
                        self.master_pos_hist.pop_front();
                        self.slave_pos_hist.pop_front();
                        self.master_vel_hist.pop_front();
                        self.slave_vel_hist.pop_front();
                        self.error_hist.pop_front();
                    }

                    self.time_history.push_back(state.time);
                    self.master_pos_hist.push_back(state.master_pos);
                    self.slave_pos_hist.push_back(state.slave_pos);
                    self.master_vel_hist.push_back(state.master_vel);
                    self.slave_vel_hist.push_back(state.slave_vel);
                    self.error_hist.push_back(state.error);
                }
                Message::ParamsAck(params) => {
                    self.params = params;
                }
            }
        }
    }

    fn send_cmd(&self, cmd: Command) {
        if let Some(tx) = &self.cmd_tx {
            let _ = tx.send(cmd);
        }
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_messages();

        // Top Menu Bar
        egui::TopBottomPanel::top("menu_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Настройки (Параметры)", |ui| {
                    if ui.button("Характеристики приводной системы (ПИД)").clicked() {
                        self.show_pid_window = true;
                        ui.close_menu();
                    }
                    if ui.button("Свойства физики и каретки").clicked() {
                        self.show_physics_window = true;
                        ui.close_menu();
                    }
                    if ui.button("Симуляция работы сервоприводной системы").clicked() {
                        self.show_servo_window = true;
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("Данные", |ui| {
                    if ui.button("Данные сервопривода ведомой оси").clicked() {
                        self.show_slave_data_window = true;
                        ui.close_menu();
                    }
                });
            });
        });

        // Status Bar (below Menu Bar)
        egui::TopBottomPanel::top("status_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if self.is_connected {
                    ui.label(egui::RichText::new("🟢 Подключено к Серверу").color(egui::Color32::GREEN).strong());
                    ui.separator();
                    ui.label(format!("Время симуляции: {:.2} c", self.latest_state.time));
                    ui.separator();
                    ui.label(format!("Ошибка: {:.4}", self.latest_state.error));
                } else {
                    ui.label(egui::RichText::new("🔴 Ожидание подключения к серверу... (127.0.0.1:5000)").color(egui::Color32::RED).strong());
                }
            });
        });

        // Floating Parameter Windows
        let mut params_changed = false;

        let mut show_pid_window = self.show_pid_window;
        egui::Window::new("Характеристики приводной системы (ПИД)")
            .open(&mut show_pid_window)
            .show(ctx, |ui| {
                params_changed |= ui.add(egui::Slider::new(&mut self.params.kp, 0.0..=500.0).text("Kp (Пропорциональный)")).changed();
                params_changed |= ui.add(egui::Slider::new(&mut self.params.ki, 0.0..=50.0).text("Ki (Интегральный)")).changed();
                params_changed |= ui.add(egui::Slider::new(&mut self.params.kd, 0.0..=50.0).text("Kd (Дифференциальный)")).changed();
            });
        self.show_pid_window = show_pid_window;

        let mut show_physics_window = self.show_physics_window;
        egui::Window::new("Свойства физики и каретки")
            .open(&mut show_physics_window)
            .show(ctx, |ui| {
                params_changed |= ui.add(egui::Slider::new(&mut self.params.mass, 0.1..=50.0).text("Масса каретки/нагрузки (kg)")).changed();
                params_changed |= ui.add(egui::Slider::new(&mut self.params.friction, 0.0..=10.0).text("Трение в механизме")).changed();
                params_changed |= ui.add(egui::Slider::new(&mut self.params.gear_ratio, 0.1..=10.0).text("Передаточное число")).changed();
            });
        self.show_physics_window = show_physics_window;
            
        let mut show_servo_window = self.show_servo_window;
        egui::Window::new("Параметры сервопривода и мотора")
            .open(&mut show_servo_window)
            .show(ctx, |ui| {
                params_changed |= ui.add(egui::Slider::new(&mut self.params.motor_inertia, 0.0..=1.0).text("Момент инерции ротора (kg*m^2)")).changed();
                params_changed |= ui.add(egui::Slider::new(&mut self.params.motor_torque_const, 0.1..=10.0).text("Постоянная момента Kt (N*m/A)")).changed();
                params_changed |= ui.add(egui::Slider::new(&mut self.params.drive_max_current, 1.0..=100.0).text("Максимальный ток привода (A)")).changed();
            });
        self.show_servo_window = show_servo_window;
            
        let mut show_slave_data_window = self.show_slave_data_window;
        egui::Window::new("Данные сервопривода ведомой оси")
            .open(&mut show_slave_data_window)
            .show(ctx, |ui| {
                ui.heading("Выдаваемые данные (Телеметрия):");
                ui.label(format!("Текущая позиция: {:.3}", self.latest_state.slave_pos));
                ui.label(format!("Текущая скорость: {:.3}", self.latest_state.slave_vel));
                ui.label(format!("Текущее ускорение: {:.3}", self.latest_state.slave_acc));
                
                ui.separator();
                
                ui.heading("Принимаемые данные (Управление):");
                ui.checkbox(&mut self.pos_mode, "Включить прием заданной позиции (Абсолютный режим)");
                if self.pos_mode {
                    ui.horizontal(|ui| {
                        ui.label("Заданная позиция:");
                        if ui.add(egui::DragValue::new(&mut self.target_pos).speed(0.1)).changed() {
                            self.send_cmd(Command::SetSlaveTargetPos(Some(self.target_pos)));
                        }
                    });
                } else {
                    ui.label(egui::RichText::new("Режим отключен. Привод синхронизирован с мастером.").color(egui::Color32::GRAY));
                    if ui.button("Принудительная синхронизация с мастером").clicked() {
                        self.send_cmd(Command::SetSlaveTargetPos(None));
                    }
                }
            });
        self.show_slave_data_window = show_slave_data_window;

        if params_changed {
            self.send_cmd(Command::UpdateParams(self.params.clone()));
        }

        // Left Control Panel (Motion only)
        egui::SidePanel::left("controls").show(ctx, |ui| {
            ui.heading("Мастер-ось");
            ui.separator();

            if ui.add(egui::Slider::new(&mut self.target_vel, -100.0..=100.0).text("Целевая скорость мастера")).changed() {
                self.send_cmd(Command::SetMasterTargetVel(self.target_vel));
            }

            ui.add_space(20.0);
            ui.separator();
            if ui.button("Сброс симуляции").clicked() {
                self.send_cmd(Command::Reset);
            }
        });

        // Main Chart Panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Графики движения");

            let time = &self.time_history;

            let master_pos: PlotPoints = time.iter().zip(self.master_pos_hist.iter()).map(|(x, y)| [*x, *y]).collect();
            let slave_pos: PlotPoints = time.iter().zip(self.slave_pos_hist.iter()).map(|(x, y)| [*x, *y]).collect();
            
            let master_vel: PlotPoints = time.iter().zip(self.master_vel_hist.iter()).map(|(x, y)| [*x, *y]).collect();
            let slave_vel: PlotPoints = time.iter().zip(self.slave_vel_hist.iter()).map(|(x, y)| [*x, *y]).collect();

            let error: PlotPoints = time.iter().zip(self.error_hist.iter()).map(|(x, y)| [*x, *y]).collect();

            ui.columns(1, |cols| {
                cols[0].label("Позиция");
                Plot::new("PosPlot")
                    .height(200.0)
                    .show(&mut cols[0], |plot_ui| {
                        plot_ui.line(Line::new(master_pos).name("Мастер"));
                        plot_ui.line(Line::new(slave_pos).name("Слейв"));
                    });

                cols[0].label("Скорость");
                Plot::new("VelPlot")
                    .height(200.0)
                    .show(&mut cols[0], |plot_ui| {
                        plot_ui.line(Line::new(master_vel).name("Мастер"));
                        plot_ui.line(Line::new(slave_vel).name("Слейв"));
                    });

                cols[0].label("Ошибка рассогласования");
                Plot::new("ErrPlot")
                    .height(150.0)
                    .show(&mut cols[0], |plot_ui| {
                        plot_ui.line(Line::new(error).name("Ошибка"));
                    });
            });
        });

        ctx.request_repaint(); // Continuous repaint for smooth charts
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Servo Motion Simulation Client",
        options,
        Box::new(|_cc| Ok(Box::new(AppState::new(_cc)))),
    )
}
