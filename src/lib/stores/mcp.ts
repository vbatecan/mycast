import { get, writable } from 'svelte/store';
import { invoke, isTauri } from '@tauri-apps/api/core';

export interface McpServerConfig {
  id: string;
  name: string;
  command: string;
  args: string[];
}
export interface McpServer {
  config: McpServerConfig;
  connected: boolean;
  tools: string[];
}
export interface ToolApproval {
  id: string;
  server: string;
  tool: string;
  arguments: Record<string, unknown>;
}
export const mcpServers = writable<McpServer[]>([]);
export const pendingToolApproval = writable<ToolApproval | null>(null);

async function update(command: string, args?: Record<string, unknown>) {
  if (!isTauri())
    throw new Error('MCP connections are available in the desktop app.');
  const next = await invoke<McpServer[]>(command, args);
  mcpServers.set(next);
}
export const loadMcpServers = () => update('get_mcp_servers');
export const saveMcpServer = (config: McpServerConfig) =>
  update('save_mcp_server', { config });
export const removeMcpServer = (id: string) =>
  update('remove_mcp_server', { id });
export const connectMcpServer = (id: string) =>
  update('connect_mcp_server', { id });
export const disconnectMcpServer = (id: string) =>
  update('disconnect_mcp_server', { id });
export async function approveTool(allow: boolean) {
  const pending = get(pendingToolApproval);
  if (!pending) return;
  try {
    await invoke('mcp_approve_tool', { id: pending.id, allow });
  } finally {
    if (get(pendingToolApproval)?.id === pending.id)
      pendingToolApproval.set(null);
  }
}
