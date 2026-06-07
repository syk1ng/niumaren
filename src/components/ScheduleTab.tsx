import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Schedule } from '../types';
import { RefreshCw } from 'lucide-react';

export default function ScheduleTab() {
  const [schedules, setSchedules] = useState<Schedule[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const data = await invoke<Schedule[]>('get_schedules');
      setSchedules(data);
      setError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { load(); }, [load]);

  const getStatusBadge = (s: Schedule) => {
    if (s.is_holiday === 1) {
      return <span className="text-xs bg-gray-800 text-gray-500 px-2 py-0.5 rounded">已跳过（节假日）</span>;
    }
    if (s.notified === 1) {
      return <span className="text-xs bg-green-900/50 text-green-400 px-2 py-0.5 rounded">✅ 已通知</span>;
    }
    return <span className="text-xs bg-amber-900/50 text-amber-400 px-2 py-0.5 rounded">⏳ 待发送</span>;
  };

  if (loading) return <div className="text-gray-400">加载中...</div>;

  return (
    <div>
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-sm font-semibold text-gray-300">排班记录</h2>
        <button onClick={load} className="btn-secondary flex items-center gap-1 text-xs">
          <RefreshCw size={12} /> 刷新
        </button>
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
              <th className="py-2 px-3">日期</th>
              <th className="py-2 px-3">星期</th>
              <th className="py-2 px-3">值班人</th>
              <th className="py-2 px-3">状态</th>
              <th className="py-2 px-3">通知时间</th>
            </tr>
          </thead>
          <tbody>
            {schedules.map(s => {
              const date = new Date(s.duty_date);
              const weekdays = ['日', '一', '二', '三', '四', '五', '六'];
              const weekday = weekdays[date.getDay()];
              return (
                <tr key={s.id} className={`border-b border-gray-800 ${s.is_holiday ? 'opacity-40' : ''}`}>
                  <td className="py-2 px-3">{s.duty_date}</td>
                  <td className="py-2 px-3">
                    <span className={date.getDay() === 0 || date.getDay() === 6 ? 'text-amber-400' : ''}>
                      周{weekday}
                    </span>
                  </td>
                  <td className="py-2 px-3">{s.person_name || '-'}</td>
                  <td className="py-2 px-3">{getStatusBadge(s)}</td>
                  <td className="py-2 px-3 text-gray-500 text-xs">
                    {s.notified_at || '-'}
                  </td>
                </tr>
              );
            })}
            {schedules.length === 0 && (
              <tr>
                <td colSpan={5} className="text-center py-8 text-gray-500">
                  暂无排班记录。应用运行后将自动生成排班。
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
