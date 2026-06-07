import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { EmailLog } from '../types';
import { RefreshCw } from 'lucide-react';

type FilterStatus = 'all' | 'success' | 'failed';

export default function LogTab() {
  const [logs, setLogs] = useState<EmailLog[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filter, setFilter] = useState<FilterStatus>('all');

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const data = await invoke<EmailLog[]>('get_email_logs');
      setLogs(data);
      setError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { load(); }, [load]);

  const filtered = filter === 'all'
    ? logs
    : logs.filter(l => l.status === filter);

  if (loading) return <div className="text-gray-400">加载中...</div>;

  return (
    <div>
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-sm font-semibold text-gray-300">邮件发送记录</h2>
        <div className="flex gap-2">
          <div className="flex rounded-lg overflow-hidden border border-gray-700 text-xs">
            {(['all', 'success', 'failed'] as FilterStatus[]).map(f => (
              <button
                key={f}
                onClick={() => setFilter(f)}
                className={`px-3 py-1 ${
                  filter === f ? 'bg-amber-600 text-white' : 'bg-gray-800 text-gray-400 hover:bg-gray-700'
                }`}
              >
                {f === 'all' ? '全部' : f === 'success' ? '成功' : '失败'}
              </button>
            ))}
          </div>
          <button onClick={load} className="btn-secondary flex items-center gap-1 text-xs">
            <RefreshCw size={12} /> 刷新
          </button>
        </div>
      </div>

      {error && (
        <div className="bg-red-900/50 border border-red-700 text-red-200 px-4 py-2 rounded-lg mb-4 text-sm">
          {error}
        </div>
      )}

      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-gray-700 text-gray-400 text-left">
              <th className="py-2 px-3">时间</th>
              <th className="py-2 px-3">收件人</th>
              <th className="py-2 px-3">邮件标题</th>
              <th className="py-2 px-3">状态</th>
              <th className="py-2 px-3">错误信息</th>
            </tr>
          </thead>
          <tbody>
            {filtered.map(log => (
              <tr key={log.id} className={`border-b border-gray-800 ${
                log.status === 'failed' ? 'bg-red-900/10' : ''
              }`}>
                <td className="py-2 px-3 text-gray-500 text-xs">{log.sent_at}</td>
                <td className="py-2 px-3">{log.recipient}</td>
                <td className="py-2 px-3 text-gray-300 max-w-60 truncate">{log.subject}</td>
                <td className="py-2 px-3">
                  {log.status === 'success' ? (
                    <span className="text-green-400 text-xs">✅ 成功</span>
                  ) : (
                    <span className="text-red-400 text-xs">❌ 失败</span>
                  )}
                </td>
                <td className="py-2 px-3 text-red-400 text-xs max-w-40 truncate">
                  {log.error_msg || '-'}
                </td>
              </tr>
            ))}
            {filtered.length === 0 && (
              <tr>
                <td colSpan={5} className="text-center py-8 text-gray-500">
                  暂无发送记录
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
