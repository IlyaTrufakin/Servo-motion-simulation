import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from './ui/card';
import { Label } from './ui/label';
import { Input } from './ui/input';
import { Slider } from './ui/slider';
import { ServoParams } from '../types';
import { Separator } from './ui/separator';
import { Button } from './ui/button';
import { Play, Pause, RotateCcw, Target, Zap } from 'lucide-react';

interface ServoControlsProps {
  params: ServoParams;
  onParamsChange: (params: Partial<ServoParams>) => void;
  masterVel: number;
  onMasterVelChange: (vel: number) => void;
  slaveTarget: number | null;
  onSlaveTargetChange: (pos: number | null) => void;
  isRunning: boolean;
  onToggleRunning: () => void;
  onReset: () => void;
}

export const ServoControls: React.FC<ServoControlsProps> = ({
  params,
  onParamsChange,
  masterVel,
  onMasterVelChange,
  slaveTarget,
  onSlaveTargetChange,
  isRunning,
  onToggleRunning,
  onReset,
}) => {
  return (
    <div className="flex flex-col gap-5 h-full">
      <div>
        <span className="section-title">Mechanical Params</span>
        <div className="space-y-4">
          <div className="space-y-2">
            <div className="flex justify-between text-xs">
              <span className="text-muted-foreground">Mass (m)</span>
              <span className="font-mono text-primary">{params.mass} kg</span>
            </div>
            <Slider 
              value={[params.mass]} 
              min={0.1} max={50} step={0.1}
              onValueChange={(v: any) => onParamsChange({ mass: Array.isArray(v) ? v[0] : v })}
            />
          </div>
          <div className="space-y-2">
            <div className="flex justify-between text-xs">
              <span className="text-muted-foreground">Friction (μ)</span>
              <span className="font-mono text-primary">{params.friction}</span>
            </div>
            <Slider 
              value={[params.friction]} 
              min={0} max={10} step={0.1}
              onValueChange={(v: any) => onParamsChange({ friction: Array.isArray(v) ? v[0] : v })}
            />
          </div>
        </div>
      </div>

      <div>
        <span className="section-title">PID Control</span>
        <div className="grid grid-cols-3 gap-2">
          <div className="space-y-1">
            <Label className="text-[9px] text-muted-foreground uppercase">Kp</Label>
            <Input 
              type="number" 
              value={params.kp} 
              onChange={(e) => onParamsChange({ kp: parseFloat(e.target.value) || 0 })}
              className="h-7 text-[11px] bg-background border-border"
            />
          </div>
          <div className="space-y-1">
            <Label className="text-[9px] text-muted-foreground uppercase">Ki</Label>
            <Input 
              type="number" 
              value={params.ki} 
              onChange={(e) => onParamsChange({ ki: parseFloat(e.target.value) || 0 })}
              className="h-7 text-[11px] bg-background border-border"
            />
          </div>
          <div className="space-y-1">
            <Label className="text-[9px] text-muted-foreground uppercase">Kd</Label>
            <Input 
              type="number" 
              value={params.kd} 
              onChange={(e) => onParamsChange({ kd: parseFloat(e.target.value) || 0 })}
              className="h-7 text-[11px] bg-background border-border"
            />
          </div>
        </div>
      </div>

      <div>
        <span className="section-title">Target Motion</span>
        <div className="space-y-4">
          <div className="space-y-2">
            <div className="flex justify-between text-xs">
              <span className="text-muted-foreground">Velocity (v)</span>
              <span className="font-mono text-primary">{masterVel.toFixed(1)} m/s</span>
            </div>
            <Slider 
              value={[masterVel]} 
              min={-10} max={10} step={0.1}
              onValueChange={(v: any) => onMasterVelChange(Array.isArray(v) ? v[0] : v)}
            />
          </div>
          
          <div className="space-y-2">
            <span className="text-[10px] text-muted-foreground uppercase tracking-wider">Sync Mode</span>
            <div className="flex gap-2">
              <Button 
                variant={slaveTarget === null ? "default" : "outline"} 
                size="sm" 
                className="flex-1 text-[10px] h-8"
                onClick={() => onSlaveTargetChange(null)}
              >
                FOLLOW MASTER
              </Button>
              <Button 
                variant={slaveTarget !== null ? "default" : "outline"} 
                size="sm" 
                className="flex-1 text-[10px] h-8"
                onClick={() => onSlaveTargetChange(0)}
              >
                INDEPENDENT
              </Button>
            </div>
          </div>

          {slaveTarget === null ? (
            <div className="space-y-2">
              <div className="flex justify-between text-xs">
                <span className="text-muted-foreground">Gear Ratio</span>
                <span className="font-mono text-primary">{params.gearRatio.toFixed(2)}</span>
              </div>
              <Slider 
                value={[params.gearRatio]} 
                min={-2} max={2} step={0.01}
                onValueChange={(v: any) => onParamsChange({ gearRatio: Array.isArray(v) ? v[0] : v })}
              />
            </div>
          ) : (
            <div className="space-y-2">
              <div className="flex justify-between text-xs">
                <span className="text-muted-foreground">Target Pos</span>
                <span className="font-mono text-primary">{slaveTarget.toFixed(1)} m</span>
              </div>
              <Slider 
                value={[slaveTarget]} 
                min={-20} max={20} step={0.1}
                onValueChange={(v: any) => onSlaveTargetChange(Array.isArray(v) ? v[0] : v)}
              />
            </div>
          )}
        </div>
      </div>

      <div className="mt-auto flex flex-col gap-2">
        <Button 
          className="w-full bg-primary text-primary-foreground hover:bg-primary/90 font-bold text-xs h-10 uppercase tracking-widest"
          onClick={onToggleRunning}
        >
          {isRunning ? "Stop Simulation" : "Execute Sync Pulse"}
        </Button>
        <Button 
          variant="outline" 
          className="w-full border-border text-foreground hover:bg-muted font-bold text-xs h-10 uppercase tracking-widest"
          onClick={onReset}
        >
          Reset Simulation
        </Button>
      </div>
    </div>
  );
};
