import { ServoParams, SimulationState } from "../types";

export class ServoSimulator {
  private state: SimulationState;
  private params: ServoParams;
  private integralError: number = 0;
  private lastError: number = 0;
  private dt: number = 0.016; // ~60fps

  constructor(params: ServoParams) {
    this.params = params;
    this.state = this.getInitialState();
  }

  private getInitialState(): SimulationState {
    return {
      time: 0,
      masterPos: 0,
      masterVel: 0,
      masterAcc: 0,
      slavePos: 0,
      slaveVel: 0,
      slaveAcc: 0,
      targetPos: 0,
      error: 0,
    };
  }

  public updateParams(params: Partial<ServoParams>) {
    this.params = { ...this.params, ...params };
  }

  public reset() {
    this.state = this.getInitialState();
    this.integralError = 0;
    this.lastError = 0;
  }

  public step(masterTargetVel: number, slaveTargetPos: number | null): SimulationState {
    const { dt } = this;
    
    // 1. Update Master Axis (Simple velocity control for simulation)
    const prevMasterVel = this.state.masterVel;
    this.state.masterVel = masterTargetVel;
    this.state.masterAcc = (this.state.masterVel - prevMasterVel) / dt;
    this.state.masterPos += this.state.masterVel * dt;

    // 2. Determine Slave Target
    // If slaveTargetPos is provided, we are in "Positioning" mode
    // Otherwise, we are in "Sync" mode (following master with gearRatio)
    let desiredSlavePos: number;
    if (slaveTargetPos !== null) {
      desiredSlavePos = slaveTargetPos;
    } else {
      desiredSlavePos = this.state.masterPos * this.params.gearRatio;
    }

    // 3. PID Control for Slave
    const error = desiredSlavePos - this.state.slavePos;
    this.integralError += error * dt;
    const derivative = (error - this.lastError) / dt;
    
    let controlForce = (error * this.params.kp) + 
                       (this.integralError * this.params.ki) + 
                       (derivative * this.params.kd);

    // Limit force
    controlForce = Math.max(-this.params.maxForce, Math.min(this.params.maxForce, controlForce));

    // 4. Physics (F = ma -> a = F/m)
    // Simple friction: F_net = F_control - friction * velocity
    const frictionForce = this.params.friction * this.state.slaveVel;
    const netForce = controlForce - frictionForce;
    const acceleration = netForce / this.params.mass;

    // Integrate
    const prevSlaveVel = this.state.slaveVel;
    this.state.slaveVel += acceleration * dt;
    this.state.slaveAcc = (this.state.slaveVel - prevSlaveVel) / dt;
    this.state.slavePos += this.state.slaveVel * dt;

    this.state.time += dt;
    this.state.error = error;
    this.state.targetPos = desiredSlavePos;
    this.lastError = error;

    return { ...this.state };
  }

  public getState() {
    return { ...this.state };
  }
}
