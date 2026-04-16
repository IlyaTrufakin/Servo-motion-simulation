import React from 'react';
import { ResponsiveContainer, LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend } from 'recharts';
import { DataPoint } from '../types';

interface ServoChartsProps {
  data: DataPoint[];
}

export const ServoCharts: React.FC<ServoChartsProps> = ({ data }) => {
  return (
    <>
      {/* Velocity Chart */}
      <div className="border border-border bg-[#121217] rounded p-3 flex flex-col overflow-hidden">
        <h3 className="text-[11px] text-muted-foreground mb-2">Velocity Profile (mm/s)</h3>
        <div className="flex-1 min-h-0">
          <ResponsiveContainer width="100%" height="100%">
            <LineChart data={data}>
              <CartesianGrid strokeDasharray="3 3" stroke="#2d2d35" vertical={false} />
              <XAxis dataKey="time" hide />
              <YAxis stroke="#555" fontSize={9} />
              <Tooltip 
                contentStyle={{ backgroundColor: '#1a1a1f', border: '1px solid #2d2d35', fontSize: '10px' }}
                itemStyle={{ fontSize: '10px' }}
              />
              <Line type="monotone" dataKey="masterVel" stroke="#ff922b" dot={false} strokeWidth={1} strokeDasharray="4 4" name="Master" />
              <Line type="monotone" dataKey="slaveVel" stroke="#4dabf7" dot={false} strokeWidth={2} name="Slave" />
            </LineChart>
          </ResponsiveContainer>
        </div>
      </div>

      {/* Acceleration Chart */}
      <div className="border border-border bg-[#121217] rounded p-3 flex flex-col overflow-hidden">
        <h3 className="text-[11px] text-muted-foreground mb-2">Acceleration (mm/s²)</h3>
        <div className="flex-1 min-h-0">
          <ResponsiveContainer width="100%" height="100%">
            <LineChart data={data}>
              <CartesianGrid strokeDasharray="3 3" stroke="#2d2d35" vertical={false} />
              <XAxis dataKey="time" hide />
              <YAxis stroke="#555" fontSize={9} />
              <Tooltip 
                contentStyle={{ backgroundColor: '#1a1a1f', border: '1px solid #2d2d35', fontSize: '10px' }}
                itemStyle={{ fontSize: '10px' }}
              />
              <Line type="stepAfter" dataKey="slaveAcc" stroke="#63e6be" dot={false} strokeWidth={2} name="Slave Acc" />
            </LineChart>
          </ResponsiveContainer>
        </div>
      </div>
    </>
  );
};
