/* @layer renderer-app @kind test */
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { apiRequest } from '../../src/data/api';
import type { AccountsSwitch } from '../../src/data/types';
import { ProcessActions } from '../../src/screens/overview/accounts/switch/sub-components/ProcessActions';
import { ProcessWizard } from '../../src/screens/overview/accounts/switch/sub-components/ProcessWizard';

vi.mock('../../src/data/api', async (original) => ({ ...(await original<object>()), apiRequest: vi.fn().mockResolvedValue({}) }));
vi.mock('@drizztdourden08/tessera', async (original) => ({ ...(await original<object>()), toast: vi.fn() }));

const doc: AccountsSwitch = {
  phase: 'handover',
  to: 'b',
  steps: [
    { stage: 'detect', state: 'done', text: 'Found.' },
    { stage: 'handover', state: 'running', text: 'Asked.' },
  ],
  threads: [{ board: 'rotp', thread: 'lead', title: 'Lead thread', state: 'asked' }],
};

afterEach(() => {
  cleanup();
  vi.mocked(apiRequest).mockClear();
});

describe('the process wizard of an account switch', { tags: ['critical'] }, () => {
  it('shows the stepper and the body of the current step', () => {
    render(<ProcessWizard doc={doc} />);
    expect(screen.getAllByText('Collect handovers').length).toBeGreaterThan(0);
    expect(screen.getByText('Lead thread')).toBeTruthy();
  });

  it('posts the thread when its Skip button is clicked', () => {
    render(<ProcessWizard doc={doc} />);
    fireEvent.click(screen.getByRole('button', { name: 'Skip' }));
    expect(apiRequest).toHaveBeenCalledWith('/api/accounts/switch/skip-thread', { method: 'POST', json: { board: 'rotp', thread: 'lead' } });
  });

  it('posts the current stage when Skip this step is clicked', () => {
    render(<ProcessActions doc={doc} />);
    fireEvent.click(screen.getByRole('button', { name: 'Skip this step' }));
    expect(apiRequest).toHaveBeenCalledWith('/api/accounts/switch/skip-step', { method: 'POST', json: { stage: 'handover' } });
  });

  it('offers no step skip while the sync runs', () => {
    render(<ProcessActions doc={{ phase: 'sync', steps: [{ stage: 'sync', state: 'running' }] }} />);
    expect(screen.queryByRole('button', { name: 'Skip this step' })).toBeNull();
  });
});
