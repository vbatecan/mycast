import { test, expect } from '@playwright/test';
import { mockAiConnection } from './helpers/ai-connection';

for (const allow of [true, false])
  test(`MCP configuration and ${allow ? 'approval' : 'denial'} flow`, async ({
    page,
  }) => {
    await page.goto('http://127.0.0.1:1422');
    await mockAiConnection(page);
    await page.evaluate(() => {
      const internals = (window as any).__TAURI_INTERNALS__;
      const originalInvoke = internals.invoke;
      const originalCallback = internals.transformCallback;
      const channels = new Map();
      let servers: any[] = [];
      let finish: (() => void) | undefined;
      Object.assign(window, { toolDecisions: [], toolRuns: 0 });
      internals.transformCallback = (callback: any) => {
        const id = originalCallback(callback);
        channels.set(id, callback);
        return id;
      };
      internals.invoke = async (command: string, args: any) => {
        if (command === 'get_mcp_servers') return servers;
        if (command === 'save_mcp_server') {
          servers = [
            {
              config: { ...args.config, id: 'fixture' },
              connected: false,
              tools: [],
            },
          ];
          return servers;
        }
        if (command === 'connect_mcp_server') {
          servers[0] = { ...servers[0], connected: true, tools: ['echo'] };
          return structuredClone(servers);
        }
        if (command === 'disconnect_mcp_server') {
          servers[0] = { ...servers[0], connected: false, tools: [] };
          return structuredClone(servers);
        }
        if (command === 'ask_ai') {
          const emit = channels.get(args.output.id);
          emit({
            index: 0,
            message: {
              type: 'tool-approval',
              text: {
                id: 'approval',
                server: 'Fixture',
                tool: 'echo',
                arguments: { message: 'orchid' },
              },
            },
          });
          return new Promise<void>((resolve) => {
            finish = () => {
              emit({
                index: 1,
                message: { type: 'text', text: 'Tool decision handled.' },
              });
              resolve();
            };
          });
        }
        if (command === 'mcp_approve_tool') {
          (window as any).toolDecisions.push(args.allow);
          if (args.allow) (window as any).toolRuns++;
          finish?.();
          return;
        }
        return originalInvoke(command, args);
      };
    });
    await page.getByRole('button', { name: 'Open Mycast settings' }).click();
    await page.getByRole('tab', { name: 'MCP', exact: true }).click();
    await page.getByRole('button', { name: 'Add server' }).click();
    await page.getByLabel('Name', { exact: true }).fill('Fixture');
    await page.getByLabel('Executable', { exact: true }).fill('node');
    await page.getByLabel('Arguments', { exact: true }).fill('["fixture.mjs"]');
    await page.getByRole('button', { name: 'Save server' }).click();
    await expect(page.getByText('Disconnected', { exact: true })).toBeVisible();
    await page.getByRole('button', { name: 'Connect', exact: true }).click();
    await expect(page.getByText('Connected', { exact: true })).toBeVisible();
    await expect(page.getByText('1 tool', { exact: true })).toBeVisible();
    if (allow)
      await page.screenshot({
        path: 'artifacts/mcp-connected.png',
        animations: 'disabled',
      });
    await page
      .getByRole('button', { name: 'Back to commands', exact: true })
      .click();
    await page.getByRole('button', { name: 'Ask AI', exact: true }).click();
    const composer = page.getByRole('textbox', { name: 'Ask AI', exact: true });
    await composer.fill('Use echo for my plant');
    await composer.press('Enter');
    const approval = page.getByRole('region', { name: 'Tool request' });
    await expect(approval).toContainText('orchid');
    if (allow)
      await page.screenshot({
        path: 'artifacts/mcp-approval.png',
        animations: 'disabled',
      });
    expect(await page.evaluate(() => (window as any).toolRuns)).toBe(0);
    await approval
      .getByRole('button', { name: allow ? 'Allow once' : 'Deny', exact: true })
      .click();
    await expect(approval).toHaveCount(0);
    await expect(
      page.getByRole('document', { name: 'AI response' }),
    ).toContainText('Tool decision handled');
    expect(await page.evaluate(() => (window as any).toolDecisions)).toEqual([
      allow,
    ]);
    expect(await page.evaluate(() => (window as any).toolRuns)).toBe(
      allow ? 1 : 0,
    );
    await page
      .getByRole('button', { name: 'MCP servers', exact: true })
      .click();
    await page.getByRole('button', { name: 'Disconnect', exact: true }).click();
    await expect(page.getByText('Disconnected', { exact: true })).toBeVisible();
  });
