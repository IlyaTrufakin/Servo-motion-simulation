
export interface ServoParams {
  mass: number;          // kg
  maxForce: number;      // N
  friction: number;      // Ns/m
  kp: number;            // Proportional gain
  ki: number;            // Integral gain
  kd: number;            // Derivative gain
  gearRatio: number;     // Slave/Master ratio
}

export interface SimulationState {
  time: number;
  masterPos: number;
  masterVel: number;
  masterAcc: number;
  slavePos: number;
  slaveVel: number;
  slaveAcc: number;
  targetPos: number;
  error: number;
}

export interface DataPoint {
  time: number;
  masterPos: number;
  slavePos: number;
  masterVel: number;
  slaveVel: number;
  masterAcc: number;
  slaveAcc: number;
  error: number;
}
