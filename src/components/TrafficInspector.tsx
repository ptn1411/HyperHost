import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";

interface LogEntry {
  time: string;
  host: string;
  method: string;
  uri: string;
  status: number;
  latency: string;
  req_body: string;
}

export function TrafficInspector() {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [selectedLog, setSelectedLog] = useState<LogEntry | null>(null);

  useEffect(() => {
    const unlisten = listen<LogEntry>("nginx_access_log", (event) => {
      setLogs((prev) => [event.payload, ...prev].slice(0, 100)); // Keep last 100 logs
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return (
    <div className="flex bg-[#0a0f1d] border border-surface-3/30 rounded-xl overflow-hidden h-[600px] shadow-lg">
      <div className={`flex-1 flex flex-col ${selectedLog ? 'border-r border-surface-3/30' : ''}`}>
        <div className="p-3 border-b border-surface-3/30 bg-[#111827] flex items-center justify-between">
          <h3 className="text-sm font-semibold text-text flex items-center gap-2">
            <svg className="w-4 h-4 text-accent" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" /></svg>
            Live HTTP Traffic
          </h3>
          <span className="flex items-center gap-2 text-xs font-mono text-text-muted">
            <span className="w-2 h-2 rounded-full bg-success animate-pulse"></span>
            Listening...
          </span>
        </div>
        <div className="flex-1 overflow-y-auto block bg-[#0a0f1d]">
          <table className="w-full text-left text-xs font-mono text-gray-300">
            <thead className="sticky top-0 bg-[#1f2937] text-gray-400">
              <tr>
                <th className="px-4 py-2 font-semibold">Method</th>
                <th className="px-4 py-2 font-semibold">Status</th>
                <th className="px-4 py-2 font-semibold">Domain</th>
                <th className="px-4 py-2 font-semibold">URI</th>
                <th className="px-4 py-2 font-semibold text-right">Time</th>
              </tr>
            </thead>
            <tbody>
              {logs.length === 0 ? (
                <tr>
                  <td colSpan={5} className="text-center py-10 text-gray-500 italic">No traffic recorded yet.</td>
                </tr>
              ) : (
                logs.map((log, i) => (
                  <tr 
                    key={i} 
                    onClick={() => setSelectedLog(log)}
                    className="border-b border-white/5 hover:bg-white/5 cursor-pointer transition-colors"
                  >
                    <td className="px-4 py-2">
                      <span className={`font-semibold ${log.method === 'GET' ? 'text-blue-400' : log.method === 'POST' ? 'text-green-400' : 'text-yellow-400'}`}>
                        {log.method}
                      </span>
                    </td>
                    <td className="px-4 py-2">
                      <span className={`${log.status < 400 ? 'text-green-400' : 'text-red-400'}`}>
                        {log.status}
                      </span>
                    </td>
                    <td className="px-4 py-2 text-gray-400">{log.host}</td>
                    <td className="px-4 py-2 truncate max-w-[200px]" title={log.uri}>{log.uri}</td>
                    <td className="px-4 py-2 text-right opacity-70">
                      {log.latency}s
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>

      {selectedLog && (
        <div className="w-1/3 bg-[#111827] flex flex-col">
          <div className="p-3 border-b border-surface-3/30 flex justify-between items-center bg-[#1f2937]">
            <h4 className="text-sm font-semibold text-gray-200">Request Details</h4>
            <button onClick={() => setSelectedLog(null)} className="text-gray-400 hover:text-white">
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" /></svg>
            </button>
          </div>
          <div className="flex-1 overflow-y-auto p-4 space-y-4 text-xs font-mono">
            <div>
              <p className="text-gray-500 mb-1">URL</p>
              <p className="text-gray-200 break-all">{selectedLog.host}{selectedLog.uri}</p>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <p className="text-gray-500 mb-1">Status</p>
                <p className={`${selectedLog.status < 400 ? 'text-green-400' : 'text-red-400'} font-bold`}>{selectedLog.status}</p>
              </div>
              <div>
                <p className="text-gray-500 mb-1">Latency</p>
                <p className="text-yellow-400">{selectedLog.latency}s</p>
              </div>
            </div>
            <div>
              <p className="text-gray-500 mb-1">Request Body</p>
              <pre className="bg-black/50 p-3 rounded border border-white/5 whitespace-pre-wrap break-all text-gray-300">
                {selectedLog.req_body === "-" || !selectedLog.req_body ? (
                  <span className="text-gray-600 italic">No body</span>
                ) : (
                  selectedLog.req_body
                )}
              </pre>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
