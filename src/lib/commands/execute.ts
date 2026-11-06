import {
  copyText,
  openWindowsCommand,
  openSearchUrl,
  executeNative,
  launchApplication,
  fileAction,
  runShellCommand,
  type LaunchMode,
} from '../native';
import { get } from 'svelte/store';
import { modules, commandEnabled } from '../stores/modules';
import type { Command } from '../types/command';

export type ExecutionResult =
  | { type: 'cancelled'; message: string }
  | { type: 'message'; message: string }
  | { type: 'view'; view: 'settings' | 'about' | 'appearance' }
  | { type: 'calculator-mode' };

/** Returns UI intent; callers retain ownership of navigation, focus, and errors. */
export async function executeCommand(
  command: Command,
  mode: LaunchMode = 'normal',
): Promise<ExecutionResult> {
  if (!commandEnabled(command, get(modules)))
    throw new Error('This module is disabled.');
  const action = command.action;
  if (mode === 'administrator') {
    if (!(
      command.canRunAsAdmin ||
      (action.type === 'native' && action.name === 'notepad')
    ))
      throw new Error('Administrator launching is unavailable for this item.');
    const result =
      action.type === 'application'
        ? await launchApplication(action.applicationId, mode)
        : action.type === 'native'
          ? await executeNative(action.name, mode)
          : null;
    if (!result)
      throw new Error('Administrator launching is unavailable for this item.');
    return {
      type: result.cancelled ? 'cancelled' : 'message',
      message: result.message,
    };
  }
  switch (action.type) {
    case 'windows':
      return { type: 'message', message: await openWindowsCommand(action.id) };
    case 'web-search':
      return { type: 'message', message: await openSearchUrl(action.url) };
    case 'file':
      return {
        type: 'message',
        message: await fileAction(action.fileId, 'open'),
      };
    case 'run':
      return {
        type: 'message',
        message: await runShellCommand(action.command),
      };
    case 'application':
      return {
        type: 'message',
        ...(await launchApplication(action.applicationId)),
      };
    case 'native':
      return { type: 'message', ...(await executeNative(action.name)) };
    case 'copy':
      await copyText(action.text);
      return { type: 'message', message: `Copied ${action.text} to clipboard` };
    case 'view':
      return action;
    case 'calculator-mode':
      return action;
  }
}
