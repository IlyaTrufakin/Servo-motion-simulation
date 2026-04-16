import React, { useEffect, useRef } from 'react';
import { SimulationState } from '../types';

interface ServoCanvasProps {
  state: SimulationState;
}

export const ServoCanvas: React.FC<ServoCanvasProps> = ({ state }) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const draw = () => {
      const { width, height } = canvas;
      ctx.clearRect(0, 0, width, height);

      // Grid
      ctx.strokeStyle = '#2d2d35';
      ctx.lineWidth = 1;
      for (let i = 0; i < width; i += 50) {
        ctx.beginPath();
        ctx.moveTo(i, 0);
        ctx.lineTo(i, height);
        ctx.stroke();
      }
      for (let i = 0; i < height; i += 50) {
        ctx.beginPath();
        ctx.moveTo(0, i);
        ctx.lineTo(width, i);
        ctx.stroke();
      }

      const centerY = height / 2;
      const scale = 50; // pixels per unit

      // Rail System (dashed line)
      ctx.strokeStyle = '#2d2d35';
      ctx.setLineDash([5, 5]);
      ctx.beginPath();
      ctx.moveTo(0, centerY + 20);
      ctx.lineTo(width, centerY + 20);
      ctx.stroke();
      ctx.setLineDash([]);

      // Master Reference (Warning color)
      const masterX = (state.masterPos * scale) % width;
      ctx.strokeStyle = '#ff922b';
      ctx.lineWidth = 2;
      ctx.globalAlpha = 0.5;
      ctx.beginPath();
      ctx.moveTo(masterX, centerY - 50);
      ctx.lineTo(masterX, centerY + 50);
      ctx.stroke();
      ctx.globalAlpha = 1.0;

      // Master Label
      ctx.fillStyle = '#ff922b';
      ctx.font = '10px Segoe UI';
      ctx.textAlign = 'center';
      ctx.fillText(`MASTER REFERENCE (${state.masterPos.toFixed(1)} mm)`, masterX, centerY - 60);

      // Slave Node (Accent color)
      const slaveX = (state.slavePos * scale) % width;
      const nodeW = 80;
      const nodeH = 40;
      
      ctx.fillStyle = '#1a1a1f';
      ctx.strokeStyle = '#4dabf7';
      ctx.lineWidth = 1;
      
      // Shadow
      ctx.shadowBlur = 12;
      ctx.shadowColor = 'rgba(0,0,0,0.5)';
      ctx.shadowOffsetY = 4;
      
      ctx.beginPath();
      ctx.roundRect(slaveX - nodeW / 2, centerY - nodeH / 2, nodeW, nodeH, 4);
      ctx.fill();
      ctx.stroke();
      
      ctx.shadowBlur = 0; // Reset shadow

      // Node Label
      ctx.fillStyle = '#4dabf7';
      ctx.font = 'bold 9px Segoe UI';
      ctx.textAlign = 'center';
      ctx.fillText('SLAVE AXIS', slaveX, centerY + 4);
    };

    draw();
  }, [state]);

  return (
    <div className="relative w-full h-full bg-[#151619] rounded-lg overflow-hidden border border-[#2a2a2a] shadow-inner">
      <canvas
        ref={canvasRef}
        width={800}
        height={400}
        className="w-full h-full"
      />
      <div className="absolute top-4 right-4 flex flex-col gap-1 text-[10px] font-mono text-zinc-500 uppercase tracking-wider">
        <div>Time: {state.time.toFixed(2)}s</div>
        <div>Master Pos: {state.masterPos.toFixed(2)}</div>
        <div>Slave Pos: {state.slavePos.toFixed(2)}</div>
        <div className={Math.abs(state.error) > 0.1 ? 'text-red-400' : 'text-green-400'}>
          Error: {state.error.toFixed(4)}
        </div>
      </div>
    </div>
  );
};
