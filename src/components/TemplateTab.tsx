import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Save, Eye, Loader2 } from 'lucide-react';

const VARIABLES = [
  { v: '{姓名}', d: '值班人员姓名' },
  { v: '{邮箱}', d: '值班人员邮箱' },
  { v: '{日期}', d: '值班日期（YYYY-MM-DD）' },
  { v: '{星期}', d: '星期几' },
  { v: '{下一位姓名}', d: '下一个值班人姓名' },
  { v: '{下一位日期}', d: '下一个值班日期' },
];

export default function TemplateTab() {
  const [subject, setSubject] = useState('');
  const [body, setBody] = useState('');
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [preview, setPreview] = useState(false);
  const [msg, setMsg] = useState<string | null>(null);

  const load = useCallback(async () => {
    try {
      const data = await invoke<{ key: string; value: string }[]>('get_settings');
      const map: Record<string, string> = {};
      data.forEach(s => { map[s.key] = s.value; });
      setSubject(map.email_subject_template || '【值班通知】{日期} {星期}');
      setBody(map.email_body_template || 'Hi {姓名}，{日期} {星期} 你值班。');
    } catch (e) {
      setMsg(`加载失败: ${e}`);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { load(); }, [load]);

  const save = async () => {
    setSaving(true);
    try {
      await invoke('save_setting', { key: 'email_subject_template', value: subject });
      await invoke('save_setting', { key: 'email_body_template', value: body });
      setMsg('模板已保存');
    } catch (e) {
      setMsg(`保存失败: ${e}`);
    } finally {
      setSaving(false);
    }
  };

  const insertVar = (v: string) => {
    const ta = document.querySelector('textarea') as HTMLTextAreaElement;
    if (!ta) return;
    const start = ta.selectionStart;
    const end = ta.selectionEnd;
    const newBody = body.substring(0, start) + v + body.substring(end);
    setBody(newBody);
    setTimeout(() => {
      ta.selectionStart = ta.selectionEnd = start + v.length;
      ta.focus();
    }, 0);
  };

  const previewVars = (text: string) => text
    .replace(/\{姓名\}/g, '张三')
    .replace(/\{邮箱\}/g, 'zhangsan@qq.com')
    .replace(/\{日期\}/g, '2026-06-14')
    .replace(/\{星期\}/g, '星期日')
    .replace(/\{下一位姓名\}/g, '李四')
    .replace(/\{下一位日期\}/g, '2026-06-20');

  if (loading) return <div className="text-gray-400">加载中...</div>;

  return (
    <div>
      {msg && (
        <div className="bg-green-900/50 border border-green-700 text-green-200 px-4 py-2 rounded-lg mb-4 text-sm">
          {msg}
          <button onClick={() => setMsg(null)} className="ml-3 underline">关闭</button>
        </div>
      )}

      <div className="flex gap-6">
        {/* Editor */}
        <div className="flex-1 space-y-4">
          <div>
            <label className="text-xs text-gray-400 mb-1 block">邮件标题模板</label>
            <input
              value={subject}
              onChange={e => setSubject(e.target.value)}
              className="w-full"
              placeholder="【值班通知】{日期} {星期}"
            />
          </div>

          <div>
            <label className="text-xs text-gray-400 mb-1 block">邮件正文模板</label>
            <textarea
              value={body}
              onChange={e => setBody(e.target.value)}
              className="w-full font-mono text-sm"
              rows={14}
              style={{ background: '#1f2937', border: '1px solid #374151', color: '#e5e7eb',
                       borderRadius: '6px', padding: '10px', resize: 'vertical' }}
            />
          </div>

          <div className="flex gap-2">
            <button onClick={save} disabled={saving} className="btn-primary flex items-center gap-2">
              {saving ? <Loader2 size={14} className="animate-spin" /> : <Save size={14} />}
              保存模板
            </button>
            <button
              onClick={() => setPreview(!preview)}
              className="btn-secondary flex items-center gap-2"
            >
              <Eye size={14} />
              {preview ? '关闭预览' : '预览'}
            </button>
          </div>
        </div>

        {/* Variable Reference + Preview */}
        <div className="w-72 space-y-4">
          <div className="bg-gray-900 rounded-lg p-4 border border-gray-800">
            <h3 className="text-sm font-semibold mb-2 text-gray-300">可用变量</h3>
            <div className="space-y-1.5">
              {VARIABLES.map(v => (
                <button
                  key={v.v}
                  onClick={() => insertVar(v.v)}
                  className="flex items-center justify-between w-full text-left px-2 py-1 rounded hover:bg-gray-800 text-sm"
                >
                  <code className="text-amber-400 text-xs">{v.v}</code>
                  <span className="text-gray-500 text-xs">{v.d}</span>
                </button>
              ))}
            </div>
          </div>

          {preview && (
            <div className="bg-gray-900 rounded-lg p-4 border border-gray-700">
              <h3 className="text-sm font-semibold mb-2 text-gray-300">预览效果</h3>
              <div className="text-xs text-gray-400 mb-2">
                标题：<span className="text-gray-200">{previewVars(subject)}</span>
              </div>
              <div className="text-xs text-gray-200 whitespace-pre-wrap bg-gray-950 p-3 rounded border border-gray-800">
                {previewVars(body)}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
