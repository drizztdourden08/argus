/* @layer renderer-app @kind test */
import { cleanup, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { TabMessage } from '../../src/data/hub';
import { renderAt } from '../helpers/render-at';

beforeEach(() => {
  vi.stubGlobal('CSS', { supports: () => false });
});
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const ofType = (sent: TabMessage[], type: TabMessage['type']) => sent.filter((message) => message.type === type);

describe('S2 board presence', { tags: ['standard'] }, () => {
  it('sends presence for the thread in the URL', async () => {
    const { sent } = renderAt('/p/rotp/?thread=coordinator');
    await waitFor(() => expect(ofType(sent, 'presence')).toHaveLength(1));
    expect(ofType(sent, 'presence')[0]).toEqual({ type: 'presence', scope: 'rotp', thread: 'coordinator' });
    expect(ofType(sent, 'unpresence')).toHaveLength(0);
  });

  it('sends unpresence when the page goes away', async () => {
    const { sent, unmount } = renderAt('/p/rotp/?thread=coordinator');
    await waitFor(() => expect(ofType(sent, 'presence')).toHaveLength(1));
    unmount();
    expect(ofType(sent, 'unpresence')).toEqual([{ type: 'unpresence', scope: 'rotp' }]);
  });

  it('sends no presence without a thread in the URL', async () => {
    const { sent } = renderAt('/p/rotp/');
    await waitFor(() => expect(sent.length).toBeGreaterThan(0));
    expect(ofType(sent, 'presence')).toHaveLength(0);
  });
});

describe('K4 board presence', { tags: ['standard'] }, () => {
  it('strips characters the server would refuse from the thread', async () => {
    const { sent } = renderAt('/p/rotp/?thread=lead%20one%2F%3Cx%3E');
    await waitFor(() => expect(ofType(sent, 'presence')).toHaveLength(1));
    expect(ofType(sent, 'presence')[0]).toMatchObject({ thread: 'leadonex' });
  });
});
