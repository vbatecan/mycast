export type NativeAction =
  | 'notepad'
  | 'calculator'
  | 'explorer'
  | 'windows-settings'
  | 'downloads'
  | 'documents'
  | 'desktop'
  | 'copy-date'
  | 'copy-time'
  | 'copy-uuid';

export type CommandCategory =
  | 'Applications'
  | 'Utilities'
  | 'Quick Actions'
  | 'Preferences'
  | 'Calculator'
  | 'Files'
  | 'Windows';

export type CommandAction =
  | { type: 'web-search'; url: string }
  | { type: 'windows'; id: string }
  | { type: 'native'; name: NativeAction }
  | { type: 'view'; view: 'settings' | 'about' | 'appearance' }
  | { type: 'calculator-mode' }
  | { type: 'application'; applicationId: string }
  | { type: 'file'; fileId: string; kind: 'file' | 'folder' }
  | { type: 'run'; command: string }
  | { type: 'copy'; text: string };

export interface Command {
  module?: 'calculator' | 'currency' | 'web-search' | 'windows';
  canRunAsAdmin?: boolean;
  id: string;
  /** Stable, non-executable identity; absent on ephemeral calculator results. */
  personalizationKey?: string;
  label: string;
  description: string;
  category: CommandCategory;
  icon: string;
  keywords: string[];
  action: CommandAction;
  /** Native file matching shares the relevance classes of command search. */
  searchRelevance?: number;
  searchQuery?: string;
}
