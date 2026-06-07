import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { SmtpSettings } from '../types';
import { Save, Send, Loader2 } from 'lucide-react';

export default function EmailConfigTab() {
  const [settings, setSettings] = useState<SmtpSettings>({
    smtp_host: '', smtp_port: 465, smtp_username: '',
    smtp_password: '', smtp_use_tls: true, sender_name: '值班系统',
  });
  const [testEmail, setTestEmail] = useState('');
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [testing, setTesting] = useState(false);
  const [msg, setMsg] = useState<{ type: 'success' | 'error'; text: string } | null>(null);

  const loadSettings = useCallback(async () => {
    try {
      const data = await invoke<{ key: string; value: string }[]>('get_settings');
      const map: Record<string, string> = {};
      data.forEach(s => { map[s.key] = s.value; });
      setSettings({
        smtp_host: map.smtp_host || '',
        smtp_port: parseInt(map.smtp_port || '465'),
        smtp_username: map.smtp_username || '',
        smtp_password: map.smtp_password || '',
        smtp_use_tls: map.smtp_use_tls !== 'false',
        sender_name: map.sender_name || '值班系统',
      });
      setTestEmail(map.smtp_username || '');
    } catch (e) {
      setMsg({ type: 'error', text: `加载失败: ${e}` });
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { loadSettings(); }, [loadSettings]);

  const save = async () => {
    setSaving(true);
    try {
      const entries = Object.entries(settings).map(([key, value]) => ({
        key, value: String(value),
      }));
      await invoke('save_settings', { settings: entries });
      setMsg({ type: 'success', text: '保存成功' });
    } catch (e) {
      setMsg({ type: 'error', text: `保存失败: ${e}` });
    } finally {
      setSaving(false);
    }
  };

  const testSend = async () => {
    if (!testEmail.trim()) return;
    setTesting(true);
    try {
      const result = await invoke<string>('test_send_email', { testEmail: testEmail.trim() });
      setMsg({ type: 'success', text: result });
    } catch (e) {
      setMsg({ type: 'error', text: `测试失败: ${e}` });
    } finally {
      setTesting(false);
    }
  };

  if (loading) return <div className="text-gray-400">加载中...</div>;

  return (
    <div className="max-w-lg">
      {msg && (
        <div className={`px-4 py-2 rounded-lg mb-4 text-sm ${
          msg.type === 'success'
            ? 'bg-green-900/50 border border-green-700 text-green-200'
            : 'bg-red-900/50 border border-red-700 text-red-200'
        }`}>
          {msg.text}
          <button onClick={() => setMsg(null)} className="ml-3 underline">关闭</button>
        </div>
      )}

      <div className="space-y-4">
        <div>
          <label className="text-xs text-gray-400 mb-1 block">SMTP 服务器地址</label>
          <input
            value={settings.smtp_host}
            onChange={e => setSettings({ ...settings, smtp_host: e.target.value })}
            placeholder="smtp.qq.com"
            className="w-full"
          />
        </div>

        <div>
          <label className="text-xs text-gray-400 mb-1 block">SMTP 端口</label>
          <input
            type="number"
            value={settings.smtp_port}
            onChange={e => setSettings({ ...settings, smtp_port: parseInt(e.target.value) || 465 })}
            className="w-full"
          />
        </div>

        <div>
          <label className="text-xs text-gray-400 mb-1 block">发件邮箱地址</label>
          <input
            value={settings.smtp_username}
            onChange={e => setSettings({ ...settings, smtp_username: e.target.value })}
            placeholder="your@qq.com"
            className="w-full"
          />
        </div>

        <div>
          <label className="text-xs text-gray-400 mb-1 block">SMTP 授权码</label>
          <input
            type="password"
            value={settings.smtp_password}
            onChange={e => setSettings({ ...settings, smtp_password: e.target.value })}
            placeholder="输入邮箱授权码（非登录密码）"
            className="w-full"
          />
        </div>

        <div>
          <label className="text-xs text-gray-400 mb-1 block">发件人显示名称</label>
          <input
            value={settings.sender_name}
            onChange={e => setSettings({ ...settings, sender_name: e.target.value })}
            className="w-full"
          />
        </div>

        <div className="flex items-center gap-3">
          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={settings.smtp_use_tls}
              onChange={e => setSettings({ ...settings, smtp_use_tls: e.target.checked })}
              className="w-4 h-4"
            />
            <span className="text-sm">使用 TLS/SSL 加密</span>
          </label>
        </div>

        <button onClick={save} disabled={saving} className="btn-primary flex items-center gap-2">
          {saving ? <Loader2 size={14} className="animate-spin" /> : <Save size={14} />}
          保存配置
        </button>

        <hr className="border-gray-700 my-6" />

        <h3 className="text-sm font-semibold text-gray-300">测试发送</h3>
        <div className="flex gap-3 items-end">
          <div className="flex-1">
            <label className="text-xs text-gray-400 mb-1 block">发送测试邮件到</label>
            <input
              value={testEmail}
              onChange={e => setTestEmail(e.target.value)}
              placeholder="test@example.com"
              className="w-full"
              onKeyDown={e => e.key === 'Enter' && testSend()}
            />
          </div>
          <button onClick={testSend} disabled={testing} className="btn-secondary flex items-center gap-2">
            {testing ? <Loader2 size={14} className="animate-spin" /> : <Send size={14} />}
            发送测试
          </button>
        </div>
      </div>
    </div>
  );
}
