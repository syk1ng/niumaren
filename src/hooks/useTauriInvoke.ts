import { invoke } from '@tauri-apps/api/core';
import { useState, useCallback } from 'react';

export function useTauriInvoke<T>(command: string) {
  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const call = useCallback(async (args?: Record<string, unknown>) => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<T>(command, args);
      setData(result);
      return result;
    } catch (e) {
      const msg = typeof e === 'string' ? e : (e as Error).message;
      setError(msg);
      return null;
    } finally {
      setLoading(false);
    }
  }, [command]);

  return { data, loading, error, call };
}
