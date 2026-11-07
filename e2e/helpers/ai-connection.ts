import type { Page } from '@playwright/test';

export async function mockAiConnection(page: Page, configured = true) {
  await page.evaluate((configured) => {
    const callbacks = new Map<number, (value: unknown) => void>();
    const configs = new Map<string, any>();
    const submitted: string[] = [];
    const requests: unknown[] = [];
    let serial = 0;
    Object.assign(window, {
      isTauri: true,
      aiSubmitted: submitted,
      aiRequests: requests,
      __TAURI_INTERNALS__: {
        transformCallback(callback: (value: unknown) => void) {
          callbacks.set(++serial, callback);
          return serial;
        },
        unregisterCallback(id: number) {
          callbacks.delete(id);
        },
        async invoke(command: string, args: any) {
          if (command === 'get_ai_config')
            return (
              configs.get(args.provider) ?? {
                provider: args.provider,
                model: configured ? 'design-model' : '',
                baseUrl: ['ollama', 'lmstudio'].includes(args.provider)
                  ? 'http://localhost:11434/v1'
                  : 'https://api.example.com/v1',
                hasKey: !['ollama', 'lmstudio'].includes(args.provider),
              }
            );
          if (command === 'save_ai_config') {
            const value = {
              ...args.config,
              hasKey: args.removeKey
                ? false
                : !!args.apiKey || args.config.hasKey,
            };
            configs.set(args.config.provider, value);
            return value;
          }
          if (command === 'ask_ai') {
            submitted.push(args.prompt);
            requests.push({
              prompt: args.prompt,
              history: structuredClone(args.history),
            });
            callbacks.get(args.output.id)?.({
              index: 0,
              message: {
                type: 'text',
                text: '## A quieter interface\n\nKeep the focus on your question.',
              },
            });
          }
          return null;
        },
      },
    });
  }, configured);
}
