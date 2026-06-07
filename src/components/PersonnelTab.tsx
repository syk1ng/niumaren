import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Personnel } from '../types';
import { Plus, Trash2, Check, X } from 'lucide-react';

export default function PersonnelTab() {
  const [personnel, setPersonnel] = useState<Personnel[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [newName, setNewName] = useState('');
  const [newEmail, setNewEmail] = useState('');
  const [editingId, setEditingId] = useState<number | null>(null);
  const [editName, setEditName] = useState('');
  const [editEmail, setEditEmail] = useState('');

  const loadPersonnel = useCallback(async () => {
    try {
      const data = await invoke<Personnel[]>('get_personnel');
      setPersonnel(data);
      setError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { loadPersonnel(); }, [loadPersonnel]);

  const addPerson = async () => {
    if (!newName.trim() || !newEmail.trim()) return;
    try {
      await invoke('add_personnel', { name: newName.trim(), email: newEmail.trim() });
      setNewName('');
      setNewEmail('');
      await loadPersonnel();
    } catch (e) {
      setError(String(e));
    }
  };

  const updatePerson = async (p: Personnel) => {
    try {
      await invoke('update_personnel', { personnel: p });
      setEditingId(null);
      await loadPersonnel();
    } catch (e) {
      setError(String(e));
    }
  };

  const deletePerson = async (id: number) => {
    try {
      await invoke('delete_personnel', { id });
      await loadPersonnel();
    } catch (e) {
      setError(String(e));
    }
  };

  const toggleActive = async (p: Personnel) => {
    await updatePerson({ ...p, active: p.active === 1 ? 0 : 1 });
  };

  if (loading) return <div className="text-gray-400">加载中...</div>;

  return (
    <div>
      {error && (
        <div className="bg-red-900/50 border border-red-700 text-red-200 px-4 py-2 rounded-lg mb-4 text-sm">
          {error}
          <button onClick={() => setError(null)} className="ml-3 underline">关闭</button>
        </div>
      )}

      {/* Add Form */}
      <div className="flex gap-3 mb-6 items-end">
        <div className="flex-1">
          <label className="text-xs text-gray-400 mb-1 block">姓名</label>
          <input
            value={newName}
            onChange={e => setNewName(e.target.value)}
            placeholder="张三"
            className="w-full"
            onKeyDown={e => e.key === 'Enter' && addPerson()}
          />
        </div>
        <div className="flex-1">
          <label className="text-xs text-gray-400 mb-1 block">邮箱</label>
          <input
            value={newEmail}
            onChange={e => setNewEmail(e.target.value)}
            placeholder="zhangsan@qq.com"
            className="w-full"
            onKeyDown={e => e.key === 'Enter' && addPerson()}
          />
        </div>
        <button onClick={addPerson} className="btn-primary flex items-center gap-1">
          <Plus size={14} /> 添加
        </button>
      </div>

      {/* Personnel Table */}
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-gray-700 text-gray-400 text-left">
              <th className="py-2 px-2 w-10">#</th>
              <th className="py-2 px-2">姓名</th>
              <th className="py-2 px-2">邮箱</th>
              <th className="py-2 px-2 w-20">状态</th>
              <th className="py-2 px-2 w-28">操作</th>
            </tr>
          </thead>
          <tbody>
            {personnel.map((p, idx) => (
              <tr key={p.id} className="border-b border-gray-800 hover:bg-gray-900/50">
                <td className="py-2 px-2 text-gray-500">{idx + 1}</td>
                <td className="py-2 px-2">
                  {editingId === p.id ? (
                    <input
                      value={editName}
                      onChange={e => setEditName(e.target.value)}
                      className="w-full text-sm"
                    />
                  ) : p.name}
                </td>
                <td className="py-2 px-2">
                  {editingId === p.id ? (
                    <input
                      value={editEmail}
                      onChange={e => setEditEmail(e.target.value)}
                      className="w-full text-sm"
                    />
                  ) : p.email}
                </td>
                <td className="py-2 px-2">
                  <button
                    onClick={() => toggleActive(p)}
                    className={`text-xs px-2 py-0.5 rounded ${
                      p.active === 1 ? 'bg-green-900/50 text-green-400' : 'bg-gray-800 text-gray-500'
                    }`}
                  >
                    {p.active === 1 ? '启用' : '禁用'}
                  </button>
                </td>
                <td className="py-2 px-2">
                  <div className="flex gap-1">
                    {editingId === p.id ? (
                      <>
                        <button
                          onClick={() => updatePerson({
                            ...p, name: editName, email: editEmail
                          })}
                          className="text-green-400 hover:text-green-300"
                        >
                          <Check size={16} />
                        </button>
                        <button
                          onClick={() => setEditingId(null)}
                          className="text-gray-400 hover:text-gray-300"
                        >
                          <X size={16} />
                        </button>
                      </>
                    ) : (
                      <>
                        <button
                          onClick={() => {
                            setEditingId(p.id!);
                            setEditName(p.name);
                            setEditEmail(p.email);
                          }}
                          className="text-amber-400 hover:text-amber-300 text-xs mr-2"
                        >
                          编辑
                        </button>
                        <button
                          onClick={() => deletePerson(p.id!)}
                          className="text-red-400 hover:text-red-300"
                        >
                          <Trash2 size={14} />
                        </button>
                      </>
                    )}
                  </div>
                </td>
              </tr>
            ))}
            {personnel.length === 0 && (
              <tr>
                <td colSpan={5} className="text-center py-8 text-gray-500">
                  还没有值班人员，点击上方添加
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
