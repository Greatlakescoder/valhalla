// src/components/ProcessMonitor.jsx
import React from 'react';
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer, // Add this line
} from 'recharts';
import { ChevronDown, ChevronRight, Cpu, Database } from "lucide-react";

const ProcessMonitor = ({ processes, loading }) => {
    if (loading) {
      return (
        <div className="min-h-screen bg-slate-900 text-slate-200 w-full">
          <div className="container mx-auto p-4">
            <h1 className="text-3xl font-bold mb-6 font-mono text-center bg-gradient-to-r from-sky-400 to-indigo-400 bg-clip-text text-transparent">
              SYS.MONITOR_v1.0
            </h1>
            <div className="border border-slate-700 rounded-lg overflow-hidden shadow-xl shadow-black/20 p-4 text-center">
              Loading process data...
            </div>
          </div>
        </div>
      );
    }
  
    if (!processes || processes.length === 0) {
      return (
        <div className="min-h-screen bg-slate-900 text-slate-200 w-full">
          <div className="container mx-auto p-4">
            <h1 className="text-3xl font-bold mb-6 font-mono text-center bg-gradient-to-r from-sky-400 to-indigo-400 bg-clip-text text-transparent">
              SYS.MONITOR_v1.0
            </h1>
            <div className="border border-slate-700 rounded-lg overflow-hidden shadow-xl shadow-black/20 p-4 text-center">
              No process data available
            </div>
          </div>
        </div>
      );
    }
  const [expandedProcesses, setExpandedProcesses] = React.useState(new Set());

  const toggleProcess = (pid) => {
    setExpandedProcesses(prev => {
      const newSet = new Set(prev);
      if (newSet.has(pid)) {
        newSet.delete(pid);
      } else {
        newSet.add(pid);
      }
      return newSet;
    });
  };

  const ProcessResourceGraph = ({ attributes }) => (
    <div className="w-full h-48 mt-4 bg-slate-900 p-4 rounded-lg border border-slate-700">
      <ResponsiveContainer width="100%" height="100%">
        <LineChart
          data={[
            {
              name: 'Resources',
              cpu: Number(attributes.CpuUsage),
              memory: Number(attributes.MemoryUsage),
              totalMemory: Number(attributes.TotalMemory || 0),
              totalCpu: Number(attributes.TotalCpu || 0),
            }
          ]}
        >
          <CartesianGrid strokeDasharray="3 3" stroke="#1e293b" />
          <XAxis dataKey="name" stroke="#94a3b8" />
          <YAxis stroke="#94a3b8" />
          <Tooltip 
            contentStyle={{ 
              backgroundColor: '#0f172a', 
              border: '1px solid #334155',
              color: '#e2e8f0',
              borderRadius: '4px',
            }} 
          />
          <Legend />
          <Line type="monotone" dataKey="cpu" stroke="#38bdf8" name="CPU Usage" strokeWidth={2} />
          <Line type="monotone" dataKey="memory" stroke="#818cf8" name="Memory Usage" strokeWidth={2} />
          <Line type="monotone" dataKey="totalMemory" stroke="#2dd4bf" name="Total Memory" strokeWidth={2} />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );

  const ProcessRow = ({ process, isParent }) => (
    <div className={`p-4 w-full ${isParent ? 'bg-slate-800' : 'bg-slate-900'} border-b border-slate-700 hover:bg-slate-700 transition-colors`}>
      <div className="flex items-center justify-between w-full">
        <div className="flex items-center gap-2">
          <span className="font-mono text-slate-200">{process.name}</span>
          <span className="text-slate-400 font-mono">PID:{process.pid}</span>
        </div>
        <div className="flex items-center gap-4 font-mono">
          <div className="flex items-center gap-1">
            <Cpu className="h-4 w-4 text-sky-400" />
            <span className="text-slate-200">{process.attributes.CpuUsage}%</span>
          </div>
          <div className="flex items-center gap-1">
            <Database className="h-4 w-4 text-indigo-400" />
            <span className="text-slate-200">{process.attributes.MemoryUsage}MB</span>
          </div>
        </div>
      </div>
    </div>
  );

  return (
    <div className="flex flex-col w-screen min-h-screen bg-slate-900 text-slate-200">
      <div className="flex-1 w-full p-6">
        <h1 className="text-3xl font-bold mb-6 font-mono text-center bg-gradient-to-r from-sky-400 to-indigo-400 bg-clip-text text-transparent">
          SYS.MONITOR_v1.0
        </h1>
        <div className="w-full border border-slate-700 rounded-lg overflow-hidden shadow-xl shadow-black/20">
          {processes.map(({ parent_process, forked_threads }) => (
            <div key={parent_process.pid} className="w-full border-b border-slate-700 last:border-b-0">
              <button 
                className="w-full text-left"
                onClick={() => toggleProcess(parent_process.pid)}
              >
                <div className="flex items-center w-full">
                  {expandedProcesses.has(parent_process.pid) ? 
                    <ChevronDown className="h-4 w-4 text-sky-400" /> : 
                    <ChevronRight className="h-4 w-4 text-sky-400" />
                  }
                  <ProcessRow process={parent_process} isParent={true} />
                </div>
              </button>
              
              {expandedProcesses.has(parent_process.pid) && (
                <>
                  <ProcessResourceGraph attributes={parent_process.attributes} />
                  <div className="ml-4 w-full">
                    {forked_threads.map(thread => (
                      <ProcessRow
                        key={thread.pid}
                        process={thread}
                        isParent={false}
                      />
                    ))}
                  </div>
                </>
              )}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

export default ProcessMonitor;