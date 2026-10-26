/* @layer renderer-app @kind test */
import { cleanup, screen, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { agent, mockApi, renderAgents } from './agents-harness';

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

describe('Agents tab: the agent anchor', () => {
  it('an A-n hash opens that agent drawer', async () => {
    mockApi();
    await renderAgents({ agents: { 'A-1': agent({ name: 'coder' }), 'A-2': agent({ name: 'checker' }) } }, () => undefined, '/p/alpha#A-2');
    const drawer = await screen.findByRole('dialog', { name: 'Agent detail' });
    expect(within(drawer).getByText('A-2')).toBeTruthy();
  });

  it('a hash of an unknown agent opens nothing', async () => {
    mockApi();
    await renderAgents({ agents: { 'A-1': agent({ name: 'coder' }) } }, () => undefined, '/p/alpha#A-9');
    expect(screen.queryByRole('dialog', { name: 'Agent detail' })).toBeNull();
  });
});
