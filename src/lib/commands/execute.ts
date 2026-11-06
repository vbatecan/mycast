import {
  copyText,
  executeNative,
  launchApplication,
  fileAction,
  runShellCommand,
} from '../native';
import type { Command } from '../types/command';

export type ExecutionResult =
  | { type: 'message'; message: string }
  | { type: 'view'; view: 'settings' | 'about' | 'appearance' }
  | { type: 'calculator-mode' };

/** Returns UI intent; callers retain ownership of navigation, focus, and errors. */
export async function executeCommand(
  command: Command,
): Promise<ExecutionResult> {
  const action = command.action;
  switch (action.type) {
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
