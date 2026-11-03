/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from './board-dom';
import { buildDoc, testDoc } from './fixtures';
import { mountTab } from './mount-tab';

cleanAfterEach();

describe('S1 handing a build to the owner', { tags: ['standard'] }, () => {
  it('Hand to me posts the build entry, and Unhand clears it', async () => {
    const { calls } = await mountTab({
      handed: 'b2',
      builds: [buildDoc('b1'), buildDoc('b2')],
      tests: [testDoc('T-1', { build: 'b1' }), testDoc('T-2', { build: 'b2' })],
    });
    fireEvent.click(await screen.findByRole('button', { name: 'Hand build b1 to you' }));
    await waitFor(() => expect(calls).toHaveLength(1));
    expect(calls[0]).toMatchObject({ url: '/api/p/rotp/entries', method: 'POST', body: { kind: 'build', handed: 'b1', author: 'human' } });
    fireEvent.click(screen.getByRole('button', { name: 'Stop handing build b2 to you' }));
    await waitFor(() => expect(calls).toHaveLength(2));
    expect(calls[1]?.body).toMatchObject({ kind: 'build', handed: null });
  });

  it('is not offered for a draft that is not handed', async () => {
    await mountTab({ builds: [buildDoc('d1', { status: 'draft' })], tests: [testDoc('T-1', { build: 'd1' })] });
    await screen.findByText('Test T-1');
    expect(screen.queryByRole('button', { name: /Hand build/ })).toBeNull();
  });
});
