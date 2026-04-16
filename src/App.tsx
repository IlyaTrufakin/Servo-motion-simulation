/**
 * @license
 * SPDX-License-Identifier: Apache-2.0
 */

import { useState, useEffect, useRef, useCallback } from 'react';
import { ServoSimulator } from './lib/simulation';
import { ServoParams, DataPoint, SimulationState } from './types';
import { ServoCanvas } from './components/ServoCanvas';
import { ServoCharts } from './components/ServoCharts';
import { ServoControls } from './components/ServoControls';
import { Tabs, TabsContent, TabsList, TabsTrigger } from './components/ui/tabs';
import { Activity, LayoutDashboard, Settings2 } from 'lucide-react';

const INITIAL_PARAMS: ServoParams = {
  mass: 5.0,
  maxForce: 500,
  friction: 1.5,
  kp: 200,
  ki: 10,
  kd: 50,
  gearRatio: 1.0,
};

export default function App() {
  const [params, setParams] = useState<ServoParams>(INITIAL_PARAMS);
  const [masterVel, setMasterVel] = useState(0);
  const [slaveTarget, setSlaveTarget] = useState<number | null>(null);
  const [isRunning, setIsRunning] = useState(false);
  const [state, setState] = useState<SimulationState>({
    time: 0,
    masterPos: 0,
    masterVel: 0,
    masterAcc: 0,
    slavePos: 0,
    slaveVel: 0,
    slaveAcc: 0,
    targetPos: 0,
    error: 0,
  });
  const [history, setHistory] = useState<DataPoint[]>([]);

  const simulatorRef = useRef<ServoSimulator>(new ServoSimulator(INITIAL_PARAMS));
  const requestRef = useRef<number>(null);

  const updateSimulation = useCallback(() => {
    if (!isRunning) return;

    const newState = simulatorRef.current.step(masterVel, slaveTarget);
    setState(newState);

    setHistory((prev) => {
      const newPoint: DataPoint = {
        time: newState.time,
        masterPos: newState.masterPos,
        slavePos: newState.slavePos,
        masterVel: newState.masterVel,
        slaveVel: newState.slaveVel,
        masterAcc: newState.masterAcc,
        slaveAcc: newState.slaveAcc,
        error: newState.error,
      };
      const nextHistory = [...prev, newPoint];
      // Keep last 200 points for performance
      return nextHistory.slice(-200);
    });

    requestRef.current = requestAnimationFrame(updateSimulation);
  }, [isRunning, masterVel, slaveTarget]);

  useEffect(() => {
    if (isRunning) {
      requestRef.current = requestAnimationFrame(updateSimulation);
    } else if (requestRef.current) {
      cancelAnimationFrame(requestRef.current);
    }
    return () => {
      if (requestRef.current) cancelAnimationFrame(requestRef.current);
    };
  }, [isRunning, updateSimulation]);

  const handleParamsChange = (newParams: Partial<ServoParams>) => {
    const updated = { ...params, ...newParams };
    setParams(updated);
    simulatorRef.current.updateParams(newParams);
  };

  const handleReset = () => {
    setIsRunning(false);
    simulatorRef.current.reset();
    setState(simulatorRef.current.getState());
    setHistory([]);
    setMasterVel(0);
    setSlaveTarget(null);
  };

  return (
    <div className="flex h-screen w-full bg-background text-foreground font-sans overflow-hidden flex-col">
      {/* Header */}
      <header className="h-12 bg-card border-b border-border flex items-center justify-between px-5 shrink-0">
        <div className="text-sm font-bold tracking-widest text-primary uppercase">
          ServoSync<span className="text-foreground">Pro</span> <span className="text-xs font-normal opacity-50 ml-1">v2.4</span>
        </div>
        <div className="status-pill">ENGINE ACTIVE</div>
      </header>

      <main className="flex-1 flex overflow-hidden">
        {/* Sidebar */}
        <aside className="w-[280px] bg-card border-r border-border p-5 flex flex-col gap-5 shrink-0 overflow-y-auto custom-scrollbar">
          <ServoControls
            params={params}
            onParamsChange={handleParamsChange}
            masterVel={masterVel}
            onMasterVelChange={setMasterVel}
            slaveTarget={slaveTarget}
            onSlaveTargetChange={setSlaveTarget}
            isRunning={isRunning}
            onToggleRunning={() => setIsRunning(!isRunning)}
            onReset={handleReset}
          />
        </aside>

        {/* Content Area */}
        <section className="flex-1 flex flex-col overflow-hidden bg-background">
          <div className="flex-1 relative p-10 flex flex-col justify-center border-b border-border">
            <span className="section-title absolute top-5 left-5">Real-time Node View</span>
            <ServoCanvas state={state} />
            <div className="mt-10 font-mono text-[11px] text-muted-foreground">
              Current Deviation: <span className={Math.abs(state.error) > 0.1 ? 'text-warning' : 'text-success'}>
                {state.error.toFixed(4)} mm
              </span>
            </div>
          </div>

          <div className="h-[280px] bg-card p-5 grid grid-cols-2 gap-5 overflow-hidden">
            <ServoCharts data={history} />
          </div>
        </section>
      </main>

      {/* Footer */}
      <footer className="h-8 bg-card border-t border-border flex items-center px-5 font-mono text-[10px] text-muted-foreground gap-6 shrink-0">
        <span>POS: {state.slavePos.toFixed(3)} mm</span>
        <span>VEL: {state.slaveVel.toFixed(2)} mm/s</span>
        <span>CPU: 4.2%</span>
        <span>RT-LATENCY: 0.15ms</span>
        <span className="ml-auto text-foreground uppercase">{isRunning ? 'RUNNING' : 'READY'}</span>
      </footer>
    </div>
  );
}
