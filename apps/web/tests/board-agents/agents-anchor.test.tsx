/* @layer renderer-app @kind test */
import { cleanup, screen, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { agent, mockApi, renderAgents } from './agents-harness';

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

describe('S1 Agents tab: the agent anchor', { tags: ['standard'] }, () => {
  it('an A-n hash opens that agent drawer', async () => {
    mockApi();
    await renderAgents({ agents: { 'A-1': agent({ name: 'coder' }), 'A-2': agent({ name: 'checker' }) } }, () => undefined, '/p/alpha#A-2');
    const drawer = await screen.findByRole('dialog', { name: 'Agent detail' });
    expect(within(drawer).getByText('A-2')).toBeTruthy();
  });
});
