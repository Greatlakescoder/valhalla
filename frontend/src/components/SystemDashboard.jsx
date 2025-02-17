import React, { useRef, useMemo, useState } from 'react';
import {
    LineChart,
    Line,
    XAxis,
    YAxis,
    CartesianGrid,
    Tooltip,
    Legend,
    ResponsiveContainer,
} from 'recharts';
import { ChevronDown, ChevronRight, Cpu, Database, HardDrive, Network, ShieldAlert } from "lucide-react";

const ProcessRow = ({ process, isParent }) => {
    const getThreatColor = (score) => {
        if (!score) return '';
        const numScore = parseInt(score);
        if (numScore <= 20) return 'border-l-4 border-l-green-500';
        if (numScore <= 40) return 'border-l-4 border-l-blue-500';
        if (numScore <= 60) return 'border-l-4 border-l-yellow-500';
        if (numScore <= 80) return 'border-l-4 border-l-orange-500';
        return 'border-l-4 border-l-red-500';
    };

    // Update to match exact attribute names
    const threatScore = process.attributes?.ThreatScore;
    const threatReason = process.attributes?.ThreatScoreReason;

    return (
        <div className={`p-4 w-full ${isParent ? 'bg-slate-800' : 'bg-slate-900'} 
            ${getThreatColor(threatScore)} border-b border-slate-700 
            hover:bg-slate-700 transition-colors`}>
            <div className="flex items-center justify-between w-full">
                <div className="flex items-center gap-2">
                    <span className="font-mono text-slate-200">{process.name}</span>
                    <span className="text-slate-400 font-mono">PID:{process.pid}</span>
                    {threatScore && (
                        <span className="text-slate-400 font-mono bg-slate-700 px-2 py-1 rounded-md text-sm">
                            Score: {threatScore}
                        </span>
                    )}
                </div>
                <div className="flex items-center gap-4 font-mono">
                    <div className="flex items-center gap-1">
                        <Cpu className="h-4 w-4 text-sky-400" />
                        <span className="text-slate-200">{process.attributes?.CpuUsage || '0'}%</span>
                    </div>
                    <div className="flex items-center gap-1">
                        <Database className="h-4 w-4 text-indigo-400" />
                        <span className="text-slate-200">{process.attributes?.MemoryUsage || '0'}MB</span>
                    </div>
                    {parseInt(threatScore) > 60 && (
                        <div className="flex items-center gap-1">
                            <ShieldAlert className="h-4 w-4 text-red-400" />
                            <span className="text-slate-200 text-sm">{threatReason}</span>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
};

const MetricCard = ({ title, icon: Icon, value, subValue, color }) => (
    <div className="bg-slate-800 p-4 rounded-lg border border-slate-700">
        <div className="flex items-center gap-2 mb-2">
            <Icon className={`h-5 w-5 ${color}`} />
            <h3 className="text-slate-200 font-medium">{title}</h3>
        </div>
        <div className="font-mono">
            <div className="text-2xl text-slate-200 transition-all duration-500 ease-out">
                {value}
            </div>
            {subValue && (
                <div className="text-sm text-slate-400 transition-all duration-500 ease-out">
                    {subValue}
                </div>
            )}
        </div>
    </div>
);

const CPUChart = ({ cpus }) => {
    const data = cpus.map((cpu, index) => ({
        name: `CPU ${index}`,
        usage: cpu.usage,
        frequency: cpu.frequency / 1000, // Convert to GHz
    }));

    return (
        <div className="w-full h-64 mt-4">
            <ResponsiveContainer width="100%" height="100%">
                <LineChart data={data}>
                    <CartesianGrid strokeDasharray="3 3" stroke="#1e293b" />
                    <XAxis dataKey="name" stroke="#94a3b8" />
                    <YAxis stroke="#94a3b8" />
                    <Tooltip 
                        contentStyle={{
                            backgroundColor: '#0f172a',
                            border: '1px solid #334155',
                            borderRadius: '4px',
                        }}
                    />
                    <Legend />
                    <Line 
                        type="monotone" 
                        dataKey="usage" 
                        stroke="#38bdf8" 
                        name="Usage %" 
                        strokeWidth={2}
                        animationDuration={500}
                    />
                    <Line 
                        type="monotone" 
                        dataKey="frequency" 
                        stroke="#818cf8" 
                        name="Frequency GHz" 
                        strokeWidth={2}
                        animationDuration={500}
                    />
                </LineChart>
            </ResponsiveContainer>
        </div>
    );
};

const SystemDashboard = ({ data, loading }) => {
    // All hooks must be declared first, before any conditional returns
    const [expandedProcesses, setExpandedProcesses] = useState(new Set());
    const prevValuesRef = useRef({
        cpuUsage: 0,
        memoryUsage: 0,
        diskUsage: 0,
        networkRx: 0,
        networkTx: 0
    });
    const prevDataRef = useRef(data);

    // Smooth transition helper
    const smoothValue = (newValue, key, duration = 500) => {
        const prev = prevValuesRef.current[key];
        if (typeof newValue !== 'number' || isNaN(newValue)) return prev || 0;
        
        prevValuesRef.current[key] = newValue;
        
        if (Math.abs(newValue - prev) > 50) {
            return newValue;
        }
        
        return prev + (newValue - prev) * 0.5;
    };

    // Process toggle handler
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

    // Memoized calculations
    const avgCpuUsage = useMemo(() => {
        const newValue = data.cpu?.cpus?.reduce((acc, cpu) => acc + cpu.usage, 0) / (data.cpu?.cpus?.length || 1);
        return smoothValue(newValue, 'cpuUsage');
    }, [data.cpu]);

    const memoryUsage = useMemo(() => {
        const newValue = (data.memory?.used_memory || 0);
        return smoothValue(newValue, 'memoryUsage');
    }, [data.memory]);

    const diskUsage = useMemo(() => {
        const newValue = data.disks?.disks?.[0]?.used || 0;
        return smoothValue(newValue, 'diskUsage');
    }, [data.disks]);

    // Find main network interface (excluding lo)
    const mainInterface = useMemo(() => {
        return data.network?.interfaces?.find(i => 
            i.name !== 'lo' && (i.received > 0 || i.transmitted > 0)
        ) || data.network?.interfaces?.[0];
    }, [data.network]);

    // Network usage with smooth transitions
    const networkRx = useMemo(() => {
        return smoothValue(mainInterface?.received || 0, 'networkRx');
    }, [mainInterface]);

    const networkTx = useMemo(() => {
        return smoothValue(mainInterface?.transmitted || 0, 'networkTx');
    }, [mainInterface]);

    // Update prevDataRef
    prevDataRef.current = data;

    // Now we can do the loading check
    if (loading) {
        return (
            <div className="min-h-screen bg-slate-900 text-slate-200 w-full">
                <div className="container mx-auto p-4">
                    <h1 className="text-3xl font-bold mb-6 font-mono text-center bg-gradient-to-r from-sky-400 to-indigo-400 bg-clip-text text-transparent">
                        SYS.MONITOR_v1.0
                    </h1>
                    <div className="border border-slate-700 rounded-lg overflow-hidden shadow-xl shadow-black/20 p-4 text-center">
                        Loading system data...
                    </div>
                </div>
            </div>
        );
    }

    return (
        <div className="flex flex-col w-screen min-h-screen bg-slate-900 text-slate-200">
            <div className="flex-1 w-full p-6">
                <h1 className="text-3xl font-bold mb-6 font-mono text-center bg-gradient-to-r from-sky-400 to-indigo-400 bg-clip-text text-transparent">
                    SYS.MONITOR_v1.0
                </h1>

                {/* System Metrics Overview */}
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
                    <MetricCard
                        title="CPU Usage"
                        icon={Cpu}
                        value={`${avgCpuUsage.toFixed(1)}%`}
                        subValue={`${data.cpu?.cpus?.length || 0} cores`}
                        color="text-sky-400"
                    />
                    <MetricCard
                        title="Memory"
                        icon={Database}
                        value={`${(memoryUsage / 1024).toFixed(1)} GB`}
                        subValue={`${((memoryUsage / (data.memory?.total_memory || 1)) * 100).toFixed(1)}% of ${((data.memory?.total_memory || 0) / 1024).toFixed(1)} GB`}
                        color="text-indigo-400"
                    />
                    <MetricCard
                        title="Disk"
                        icon={HardDrive}
                        value={`${diskUsage.toFixed(1)} GB`}
                        subValue={`${((data.disks?.disks?.[0]?.usage || 0)).toFixed(1)}% of ${((data.disks?.disks?.[0]?.total || 0)).toFixed(1)} GB`}
                        color="text-emerald-400"
                    />
                    <MetricCard
                        title="Network"
                        icon={Network}
                        value={`${(networkRx / 1024 / 1024).toFixed(1)} MB`}
                        subValue={`${(networkTx / 1024 / 1024).toFixed(1)} MB TX`}
                        color="text-purple-400"
                    />
                </div>

                {/* CPU Usage Chart */}
                {data.cpu?.cpus && <CPUChart cpus={data.cpu.cpus} />}

                {/* Process List */}
                <div className="w-full border border-slate-700 rounded-lg overflow-hidden shadow-xl shadow-black/20 mt-6">
                    {data.processes?.map(({ parent_process, forked_threads }) => (
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
                                <div className="ml-4 w-full">
                                    {forked_threads.map(thread => (
                                        <ProcessRow
                                            key={thread.pid}
                                            process={thread}
                                            isParent={false}
                                        />
                                    ))}
                                </div>
                            )}
                        </div>
                    ))}
                </div>
            </div>
        </div>
    );
};

export default SystemDashboard;