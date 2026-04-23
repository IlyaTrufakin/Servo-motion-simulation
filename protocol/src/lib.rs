use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServoParams {
    pub mass: f64,
    pub friction: f64,
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
    pub gear_ratio: f64,
    
    // Servo/Motor specific parameters
    pub motor_inertia: f64,
    pub motor_torque_const: f64,
    pub drive_max_current: f64,
}

impl Default for ServoParams {
    fn default() -> Self {
        Self {
            mass: 1.0,
            friction: 0.1,
            kp: 50.0,
            ki: 0.5,
            kd: 5.0,
            gear_ratio: 1.0,
            
            motor_inertia: 0.1,
            motor_torque_const: 1.5, // Kt
            drive_max_current: 20.0, // Amps
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SimulationState {
    pub time: f64,
    pub master_pos: f64,
    pub master_vel: f64,
    pub master_acc: f64,
    pub slave_pos: f64,
    pub slave_vel: f64,
    pub slave_acc: f64,
    pub target_pos: f64,
    pub error: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    UpdateParams(ServoParams),
    SetMasterTargetVel(f64),
    SetSlaveTargetPos(Option<f64>),
    Reset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    Telemetry(SimulationState),
    ParamsAck(ServoParams),
}
