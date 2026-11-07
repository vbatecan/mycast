import type { Page } from '@playwright/test';

export async function mockFileSearch(
  page: Page,
  options: {
    roots?: string[];
    state?: 'ready' | 'indexing';
    indexed?: number;
    initialItems?: string[];
    setupFailure?: boolean;
    applications?: boolean;
    applicationFailure?: boolean;
  } = {},
) {
  await page.addInitScript((options) => {
    const pending = new Map<
      string,
      {
        resolve: (value: unknown) => void;
        reject: (reason: string) => void;
        requestId: number;
      }
    >();
    const callbacks = new Map<number, (event: unknown) => void>();
    const listeners = new Map<number, { event: string; handler: number }>();
    let nextId = 0;
    let setupFailure = options.setupFailure ?? false;
    let applicationFailure = options.applicationFailure ?? false;
    let status = {
      state: options.state ?? 'ready',
      indexed: options.indexed ?? 1,
      revision: 1,
      warnings: [],
      shell: 'PowerShell',
      preferences: {
        roots: options.roots ?? ['C:\\Documents'],
        exclusions: [],
        includeHidden: false,
        terminalDirectory: 'C:\\Documents',
      },
    };
    const item = (name: string) => ({
      id: name,
      persistentKey: 'file:v1:' + name,
      name,
      parentPath: 'C:\\Documents',
      kind: 'file',
      relevance: 1000,
    });
    Object.assign(window, {
      isTauri: true,
      hasFileSearch: (query: string) => pending.has(query),
      completeFileSearch(query: string, names: string | string[]) {
        const request = pending.get(query);
        if (!request) throw new Error('No pending search for ' + query);
        request.resolve({
          items: (Array.isArray(names) ? names : [names]).map(item),
          requestId: request.requestId,
          elapsedMs: 1,
        });
        pending.delete(query);
      },
      failFileSearch(query: string) {
        const request = pending.get(query);
        if (!request) throw new Error('No pending search for ' + query);
        request.reject('Native search error: E_TEST_FAILURE');
        pending.delete(query);
      },
      recoverFileSetup() {
        setupFailure = false;
      },
      recoverApplications() {
        applicationFailure = false;
      },
      setFileStatus(next: Partial<typeof status>) {
        status = { ...status, ...next };
        for (const [id, listener] of listeners) {
          if (listener.event === 'file-index-changed')
            callbacks.get(listener.handler)?.({
              event: listener.event,
              id,
              payload: status,
            });
        }
      },
      fileListenerCount: () =>
        [...listeners.values()].filter(
          (listener) => listener.event === 'file-index-changed',
        ).length,
      __TAURI_INTERNALS__: {
        transformCallback(callback: (event: unknown) => void) {
          const id = ++nextId;
          callbacks.set(id, callback);
          return id;
        },
        async invoke(command: string, args: any = {}) {
          switch (command) {
            case 'plugin:event|listen': {
              const id = ++nextId;
              listeners.set(id, args);
              return id;
            }
            case 'get_modules':
              return {
                files: true,
                applications: options.applications ?? false,
              };
            case 'get_launcher_status':
              return {
                platform: 'windows',
                visible: true,
                shortcut: 'Alt+Space',
                shortcutError: null,
              };
            case 'get_file_status':
              if (setupFailure)
                throw new Error('Native file setup: E_TEST_SETUP');
              return status;
            case 'list_applications':
              if (applicationFailure)
                throw new Error('Native applications: E_TEST_APPS');
              return { applications: [], warnings: [] };
            case 'resolve_files':
              return [];
            case 'search_files':
              if (!args.options.query)
                return {
                  items: (args.options.collection === 'all'
                    ? (options.initialItems ?? ['project-notes.txt'])
                    : []
                  ).map(item),
                  requestId: args.options.requestId,
                  elapsedMs: 1,
                };
              return new Promise((resolve, reject) =>
                pending.set(args.options.query, {
                  resolve,
                  reject,
                  requestId: args.options.requestId,
                }),
              );
            default:
              return null;
          }
        },
      },
      __TAURI_EVENT_PLUGIN_INTERNALS__: {
        unregisterListener(_event: string, id: number) {
          listeners.delete(id);
        },
      },
    });
  }, options);
}

export async function waitForFileSearch(page: Page, query: string) {
  await page.waitForFunction(
    (query) => (window as any).hasFileSearch(query),
    query,
  );
}
