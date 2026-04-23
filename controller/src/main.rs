#![windows_subsystem = "windows"]

use eframe::egui;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use protocol::{Command, Message, ServoParams, SimulationState};

struct ServoSimulator {
    state: SimulationState,
    params: ServoParams,
    integral_error: f64,
    last_error: f64,
    dt: f64,
}

impl ServoSimulator {
    fn new(params: ServoParams) -> Self {
        Self {
            state: SimulationState::default(),
            params,
            integral_error: 0.0,
            last_error: 0.0,
            dt: 0.016, // ~60 FPS
        }
    }

    fn reset(&mut self) {
        self.state = SimulationState::default();
        self.integral_error = 0.0;
        self.last_error = 0.0;
    }

    fn step(&mut self, master_target_vel: f64, slave_target_pos: Option<f64>) -> SimulationState {
        let dt = self.dt;

        // 1. Update Master Axis
        let prev_master_vel = self.state.master_vel;
        self.state.master_vel = master_target_vel;
        self.state.master_acc = (self.state.master_vel - prev_master_vel) / dt;
        self.state.master_pos += self.state.master_vel * dt;

        // 2. Determine Slave Target
        let desired_slave_pos = if let Some(pos) = slave_target_pos {
            pos
        } else {
            self.state.master_pos * self.params.gear_ratio
        };

        // 3. PID Control for Slave
        let error = desired_slave_pos - self.state.slave_pos;
        self.integral_error += error * dt;
        let derivative = (error - self.last_error) / dt;

        let mut control_force = (error * self.params.kp)
            + (self.integral_error * self.params.ki)
            + (derivative * self.params.kd);

        // Limit force based on drive and motor characteristics
        let max_force = self.params.motor_torque_const * self.params.drive_max_current;
        control_force = control_force.clamp(-max_force, max_force);

        // 4. Physics
        let friction_force = self.params.friction * self.state.slave_vel;
        let net_force = control_force - friction_force;
        
        // Effective mass includes the load mass and the motor's rotor inertia
        let effective_mass = self.params.mass + self.params.motor_inertia;
        let acceleration = net_force / effective_mass;

        // Integrate
        let prev_slave_vel = self.state.slave_vel;
        self.state.slave_vel += acceleration * dt;
        self.state.slave_acc = (self.state.slave_vel - prev_slave_vel) / dt;
        self.state.slave_pos += self.state.slave_vel * dt;

        self.state.time += dt;
        self.state.error = error;
        self.state.target_pos = desired_slave_pos;
        self.last_error = error;

        self.state.clone()
    }
}

// GUI Application State
struct ControllerApp {
    is_sim_running: Arc<AtomicBool>,
    connected_clients: Arc<AtomicUsize>,
    latest_state: SimulationState,
    state_rx: Receiver<SimulationState>,
    cmd_tx: Sender<Command>, // To send commands from PLC GUI to Sim Thread
}

impl ControllerApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let is_sim_running = Arc::new(AtomicBool::new(false)); // Start stopped
        let connected_clients = Arc::new(AtomicUsize::new(0));

        let (sim_cmd_tx, sim_cmd_rx) = mpsc::channel::<Command>();
        let (state_tx, state_rx) = mpsc::channel::<SimulationState>();

        let is_running_sim_clone = Arc::clone(&is_sim_running);
        let connected_clients_clone = Arc::clone(&connected_clients);

        let clients: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));

        // --- TCP Server Thread ---
        let clients_server_clone = Arc::clone(&clients);
        let sim_cmd_tx_server = sim_cmd_tx.clone();
        let connected_clients_server_clone = Arc::clone(&connected_clients_clone);
        
        thread::spawn(move || {
            let listener = TcpListener::bind("127.0.0.1:5000").expect("Failed to bind to port 5000");
            println!("Server listening on 127.0.0.1:5000");

            for stream in listener.incoming() {
                if let Ok(stream) = stream {
                    connected_clients_server_clone.fetch_add(1, Ordering::SeqCst);
                    let tx = sim_cmd_tx_server.clone();
                    let c = Arc::clone(&clients_server_clone);
                    let counter = Arc::clone(&connected_clients_server_clone);
                    
                    thread::spawn(move || {
                        handle_client(stream, tx, c, counter);
                    });
                }
            }
        });

        // --- Simulation Thread ---
        let clients_sim_clone = Arc::clone(&clients);
        thread::spawn(move || {
            let mut simulator = ServoSimulator::new(ServoParams::default());
            let mut master_target_vel = 0.0;
            let mut slave_target_pos: Option<f64> = None;

            let dt = Duration::from_millis(16);
            loop {
                let start_time = Instant::now();

                // Process commands (from GUI clients or from this PLC GUI)
                while let Ok(cmd) = sim_cmd_rx.try_recv() {
                    match cmd {
                        Command::UpdateParams(params) => {
                            simulator.params = params.clone();
                            let msg = Message::ParamsAck(params);
                            broadcast(&clients_sim_clone, &msg);
                        }
                        Command::SetMasterTargetVel(v) => master_target_vel = v,
                        Command::SetSlaveTargetPos(p) => slave_target_pos = p,
                        Command::Reset => {
                            simulator.reset();
                            master_target_vel = 0.0;
                            slave_target_pos = None;
                        }
                    }
                }

                if is_running_sim_clone.load(Ordering::SeqCst) {
                    let state = simulator.step(master_target_vel, slave_target_pos);
                    
                    // Send to internal GUI
                    let _ = state_tx.send(state.clone());

                    // Broadcast telemetry to external GUI clients
                    let msg = Message::Telemetry(state);
                    broadcast(&clients_sim_clone, &msg);
                }

                let elapsed = start_time.elapsed();
                if elapsed < dt {
                    thread::sleep(dt - elapsed);
                }
            }
        });

        Self {
            is_sim_running,
            connected_clients,
            latest_state: SimulationState::default(),
            state_rx,
            cmd_tx: sim_cmd_tx,
        }
    }
}

fn handle_client(stream: TcpStream, cmd_tx: Sender<Command>, clients: Arc<Mutex<Vec<TcpStream>>>, counter: Arc<AtomicUsize>) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    
    {
        let mut clients_lock = clients.lock().unwrap();
        clients_lock.push(stream.try_clone().unwrap());
    }

    let mut line = String::new();
    while let Ok(bytes_read) = reader.read_line(&mut line) {
        if bytes_read == 0 {
            break; // Connection closed
        }
        
        if let Ok(cmd) = serde_json::from_str::<Command>(&line) {
            let _ = cmd_tx.send(cmd);
        }
        line.clear();
    }

    // Client disconnected
    counter.fetch_sub(1, Ordering::SeqCst);
}

fn broadcast(clients: &Arc<Mutex<Vec<TcpStream>>>, msg: &Message) {
    let mut dead_clients = Vec::new();
    if let Ok(serialized) = serde_json::to_string(msg) {
        let serialized = serialized + "\n";
        let mut clients_lock = clients.lock().unwrap();

        for (i, client) in clients_lock.iter_mut().enumerate() {
            if client.write_all(serialized.as_bytes()).is_err() {
                dead_clients.push(i);
            }
        }

        // Remove disconnected clients
        for i in dead_clients.iter().rev() {
            clients_lock.remove(*i);
        }
    }
}

impl eframe::App for ControllerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Read latest state for UI
        while let Ok(state) = self.state_rx.try_recv() {
            self.latest_state = state;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Servo PLC Controller");
            ui.separator();

            let running = self.is_sim_running.load(Ordering::SeqCst);
            
            ui.horizontal(|ui| {
                ui.label("Simulation Status:");
                if running {
                    ui.label(egui::RichText::new("RUNNING").color(egui::Color32::GREEN).strong());
                } else {
                    ui.label(egui::RichText::new("STOPPED").color(egui::Color32::RED).strong());
                }
            });

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui.button(if running { "Pause" } else { "Start" }).clicked() {
                    self.is_sim_running.store(!running, Ordering::SeqCst);
                }
                
                if ui.button("Reset").clicked() {
                    let _ = self.cmd_tx.send(Command::Reset);
                    // Also reset local state display immediately
                    self.latest_state = SimulationState::default();
                }
            });

            ui.separator();
            
            let clients = self.connected_clients.load(Ordering::SeqCst);
            ui.horizontal(|ui| {
                ui.label("TCP Server:");
                ui.label(egui::RichText::new("Listening on 127.0.0.1:5000").color(egui::Color32::LIGHT_BLUE));
            });
            ui.label(format!("Connected GUI Clients: {}", clients));

            ui.separator();
            ui.label("Telemetry Preview:");
            ui.label(format!("Time: {:.2} s", self.latest_state.time));
            ui.label(format!("Master Pos: {:.2}", self.latest_state.master_pos));
            ui.label(format!("Slave Pos: {:.2}", self.latest_state.slave_pos));
            ui.label(format!("Error: {:.4}", self.latest_state.error));
        });

        // Request repaint so the time updates continuously when running
        if self.is_sim_running.load(Ordering::SeqCst) {
            ctx.request_repaint();
        } else {
            // slower repaint when stopped just to check for client connections
            ctx.request_repaint_after(Duration::from_millis(500));
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([350.0, 300.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Servo PLC Controller",
        options,
        Box::new(|cc| Ok(Box::new(ControllerApp::new(cc)))),
    )
}
