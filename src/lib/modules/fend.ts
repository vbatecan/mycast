import { invoke, isTauri } from '@tauri-apps/api/core';
export async function evaluate(expression: string): Promise<string> {
  if (!isTauri())
    throw new Error('Advanced calculations are available in the desktop app.');
  return invoke<string>('evaluate_math', { expression });
}
