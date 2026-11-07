import { get, writable } from 'svelte/store';
import { Channel, invoke } from '@tauri-apps/api/core';
import { isNative } from '../native';
import { conversationHistory } from '../ai/conversation';
import { pendingToolApproval, type ToolApproval } from './mcp';
export const providers = {
  openai: 'OpenAI',
  claude: 'Claude',
  gemini: 'Gemini',
  custom: 'Custom OpenAI-compatible',
  ollama: 'Ollama',
  lmstudio: 'LM Studio',
} as const;
export type Provider = keyof typeof providers;
export interface AiConfig {
  provider: Provider;
  model: string;
  baseUrl: string;
  hasKey: boolean;
}
let initial: Provider = 'openai';
try {
  const value = localStorage.getItem('mycast.ai.provider');
  if (value && value in providers) initial = value as Provider;
} catch {}
export const aiProvider = writable<Provider>(initial);
let lastProvider = initial;
aiProvider.subscribe((value) => {
  try {
    localStorage.setItem('mycast.ai.provider', value);
  } catch {}
  if (lastProvider !== value) {
    lastProvider = value;
    void clearAi();
  }
});
const desktop = () => {
  if (!isNative())
    throw new Error(
      'AI is available in the desktop app. Configure your provider there.',
    );
};
export async function getAiConfig(provider: Provider): Promise<AiConfig> {
  desktop();
  return invoke('get_ai_config', { provider });
}
export async function saveAiConfig(
  config: AiConfig,
  apiKey: string,
  removeKey: boolean,
): Promise<AiConfig> {
  desktop();
  return invoke('save_ai_config', {
    config,
    apiKey: apiKey || null,
    removeKey,
  });
}
export interface AiResponse {
  text: string;
  thinking: string;
  status: string;
  error: string;
  running: boolean;
  prompt: string;
}
const empty = (): AiResponse => ({
  text: '',
  thinking: '',
  status: '',
  error: '',
  running: false,
  prompt: '',
});
export const aiResponse = writable<AiResponse>(empty());
export const aiPastTurns = writable<AiResponse[]>([]);
let active = '';
let pending: Promise<void> | undefined;
let stopping: Promise<void> | undefined;
export function stopAi(): Promise<void> {
  pendingToolApproval.set(null);
  if (stopping) return stopping;
  const id = active;
  if (!id) return Promise.resolve();
  active = '';
  stopping = (async () => {
    await invoke('cancel_ai', { id }).catch(() => {});
    await pending?.catch(() => {});
    aiResponse.update((s) => ({ ...s, running: false, status: 'Stopped' }));
  })().finally(() => {
    stopping = undefined;
  });
  return stopping;
}
export async function askAi(prompt: string, retry = false) {
  if (!prompt.trim() || get(aiResponse).running || stopping) return;
  const previous = get(aiResponse);
  if (!retry && previous.prompt) {
    aiPastTurns.update((turns) => [...turns, { ...previous }].slice(-24));
  }
  const history = conversationHistory(get(aiPastTurns));
  let requestId = '';
  aiResponse.set({ ...empty(), prompt, running: true, status: 'Connecting…' });
  try {
    desktop();
    const id = crypto.randomUUID();
    active = id;
    requestId = id;
    const output = new Channel<
      | { type: 'text' | 'thinking' | 'status'; text: string }
      | { type: 'tool-approval'; text: ToolApproval }
    >();
    output.onmessage = (event) => {
      if (active !== id) return;
      if (event.type === 'tool-approval') {
        pendingToolApproval.set(event.text);
        return;
      }
      aiResponse.update((s) =>
        event.type === 'status'
          ? { ...s, status: event.text }
          : event.type === 'thinking'
            ? { ...s, thinking: s.thinking + event.text, status: 'Thinking…' }
            : { ...s, text: s.text + event.text, status: 'Writing…' },
      );
    };
    pending = invoke<void>('ask_ai', {
      id,
      provider: get(aiProvider),
      prompt,
      history,
      output,
    });
    await pending;
    if (active === id) {
      active = '';
      aiResponse.update((s) => ({
        ...s,
        running: false,
        status: s.text
          ? 'Complete'
          : 'No answer returned. Try another prompt or model.',
      }));
    }
  } catch (error) {
    if (requestId && active !== requestId) return;
    active = '';
    aiResponse.update((s) => ({
      ...s,
      running: false,
      error: String(error),
      status: 'Request failed',
    }));
  } finally {
    pendingToolApproval.set(null);
  }
}
export async function clearAi() {
  await stopAi();
  aiPastTurns.set([]);
  aiResponse.set(empty());
}
