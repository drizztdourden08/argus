/* @layer renderer-app @kind test */
import { cleanup, fireEvent, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { buildDoc, doneDoc, testDoc } from './fixtures';
import { mountTab } from './mount-tab';

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const history = [
  { status: 'fail', text: 'first try broke', by: 'human', at: '2026-10-25T09:00:00.000Z' },
  { status: 'pass', text: 'second try held', by: 'lead', at: '2026-10-26T09:00:00.000Z' },
];

const rich = () => doneDoc('T-1', 'pass', 'the sage speaks', {
  title: 'Rich test',
  covers: 'gate PedestalScenes',
  context: 'The game read the wrong level.',
  changed: ['kFeatures5 reads the hook'],
  setup: [{ label: 'profile', text: 'story-events' }, { label: 'you start', text: 'Outside the Sanctuary' }],
  run: 'rotp worktree launch story-events',
  steps: ['Walk into the Sanctuary.', 'Talk to the sage.'],
  pass: ['Zelda is there.'],
  fail: ['Zelda is gone.'],
  notes: ['Only the lead can tick I-1.'],
  ac: [1, 2],
  proofs: [
    { n: 1, caption: 'the sage at the altar', url: '/files/rotp/proofs/T-1-1.png', at: '2026-10-26T09:00:00.000Z' },
    { n: 2, caption: 'the empty altar', url: '/files/rotp/proofs/T-1-2.png', at: '2026-10-26T09:00:00.000Z' },
  ],
  runs: history,
  result: { status: 'pass', text: 'the sage speaks', by: 'lead', at: '2026-10-26T09:00:00.000Z' },
  build: 'bundle-1',
});

const items = [{ id: 'I-1', title: 'Pedestal', ac: [{ text: 'The sage speaks', done: true }, { text: 'Zelda stays', done: false }] }];

const openRich = async (extra: object = {}) => {
  const mounted = await mountTab({ builds: [buildDoc('bundle-1')], tests: [{ ...rich(), ...extra }], items, handed: 'bundle-1' });
  fireEvent.click(await screen.findByRole('button', { name: 'Rich test' }));
  const drawer = await screen.findByRole('dialog', { name: 'Test details' });
  return { ...mounted, drawer };
};

const section = (drawer: HTMLElement, name: string) => within(drawer).getByRole('group', { name });

describe('the sections of the test drawer', () => {
  it('opens from the row title with the id, the covers and the facts', async () => {
    const { drawer } = await openRich();
    expect(within(drawer).getByText('Rich test')).toBeTruthy();
    expect(within(drawer).getByText('T-1 / gate PedestalScenes')).toBeTruthy();
    expect(within(drawer).getByText('manual: needs you')).toBeTruthy();
    expect(within(drawer).getByText('handed to you')).toBeTruthy();
  });

  it('shows the result with who and the text, then steps, expected, context and what changed', async () => {
    const { drawer } = await openRich();
    expect(within(section(drawer, 'Result')).getByText('the sage speaks')).toBeTruthy();
    expect(within(section(drawer, 'Result')).getByText(/by lead/)).toBeTruthy();
    const steps = section(drawer, 'Steps');
    expect(within(steps).getByText('Walk into the Sanctuary.')).toBeTruthy();
    expect(within(steps).getByText('Talk to the sage.')).toBeTruthy();
    const expected = section(drawer, 'Expected');
    expect(within(expected).getByText('Proves it works')).toBeTruthy();
    expect(within(expected).getByText('Zelda is there.')).toBeTruthy();
    expect(within(expected).getByText('Proves it is broken')).toBeTruthy();
    expect(within(expected).getByText('Zelda is gone.')).toBeTruthy();
    expect(within(section(drawer, 'Context')).getByText('The game read the wrong level.')).toBeTruthy();
    expect(within(section(drawer, 'What changed')).getByText('kFeatures5 reads the hook')).toBeTruthy();
  });

  it('shows the setup pairs and the run command', async () => {
    const { drawer } = await openRich();
    const setup = section(drawer, 'Setup');
    expect(within(setup).getByText('profile')).toBeTruthy();
    expect(within(setup).getByText('story-events')).toBeTruthy();
    expect(within(setup).getByText('Outside the Sanctuary')).toBeTruthy();
    expect(within(setup).getByText('rotp worktree launch story-events')).toBeTruthy();
  });

  it('shows the acceptance criteria the test ticks, done or open', async () => {
    const { drawer } = await openRich();
    const covers = section(drawer, 'Covers on I-1');
    expect(within(covers).getByText('AC1: The sage speaks')).toBeTruthy();
    expect(within(covers).getByText('AC2: Zelda stays')).toBeTruthy();
    expect(within(covers).getByText('done')).toBeTruthy();
    expect(within(covers).getByText('open')).toBeTruthy();
  });

});

describe('the evidence, the history and the stream in the test drawer', () => {
  it('shows the evidence as thumbnails with their captions', async () => {
    const { drawer } = await openRich();
    const evidence = section(drawer, 'Evidence');
    expect(within(evidence).getByAltText('the sage at the altar').getAttribute('src')).toBe('/files/rotp/proofs/T-1-1.png');
    expect(within(evidence).getByAltText('the empty altar')).toBeTruthy();
    expect(within(evidence).getByRole('button', { name: 'Open the sage at the altar' })).toBeTruthy();
  });

  it('shows the results history newest first when there is more than one result', async () => {
    const { drawer } = await openRich();
    const rows = within(section(drawer, 'Results history')).getAllByRole('listitem');
    expect(rows).toHaveLength(2);
    expect(rows[0]?.textContent).toContain('second try held');
    expect(rows[1]?.textContent).toContain('first try broke');
  });

  it('shows the notes of the thread', async () => {
    const { drawer } = await openRich();
    expect(within(section(drawer, 'Notes')).getByText('Only the lead can tick I-1.')).toBeTruthy();
  });

  it('a test not run yet says so, with no history, steps or notes sections', async () => {
    await mountTab({ builds: [buildDoc('bundle-1')], tests: [testDoc('T-3', { title: 'Plain test' })] });
    fireEvent.click(await screen.findByRole('button', { name: 'Plain test' }));
    const drawer = await screen.findByRole('dialog', { name: 'Test details' });
    expect(within(section(drawer, 'Result')).getByText('Not run yet.')).toBeTruthy();
    expect(within(drawer).queryByRole('group', { name: 'Results history' })).toBeNull();
    expect(within(drawer).queryByRole('group', { name: 'Steps' })).toBeNull();
    expect(within(drawer).queryByRole('group', { name: 'Notes' })).toBeNull();
    expect(within(section(drawer, 'Evidence')).getByText('No screenshot yet.')).toBeTruthy();
  });

  it('is not in the page until a test is opened', async () => {
    await mountTab({ builds: [buildDoc('bundle-1')], tests: [testDoc('T-3', { title: 'Plain test' })] });
    await screen.findByText('Plain test');
    expect(screen.queryByRole('dialog', { name: 'Test details' })).toBeNull();
    expect(screen.queryByRole('group', { name: 'Evidence' })).toBeNull();
  });

  it('follows the stream while open: a new result shows in the drawer', async () => {
    const { drawer, change } = await openRich();
    change('tests', 'T-1', { ...rich(), result: { status: 'fail', text: 'now it broke', by: 'human', at: '2026-10-26T11:00:00.000Z' }, status: 'fail' });
    expect(await within(drawer).findByText('now it broke')).toBeTruthy();
  });
});

describe('recording from the drawer', () => {
  it('Run again is in the drawer for a done test and posts the rerun entry', async () => {
    const { drawer, calls } = await openRich();
    fireEvent.click(within(drawer).getByRole('button', { name: 'Run T-1 again' }));
    await waitFor(() => expect(calls).toHaveLength(1));
    expect(calls[0]?.body).toMatchObject({ kind: 'rerun', replyTo: 'T-1' });
  });

  it('the outcome buttons of the drawer post the result and open the same note box', async () => {
    await mountTab({ builds: [buildDoc('bundle-1')], tests: [testDoc('T-3', { title: 'Plain test' })] });
    fireEvent.click(await screen.findByRole('button', { name: 'Plain test' }));
    const drawer = await screen.findByRole('dialog', { name: 'Test details' });
    fireEvent.click(within(drawer).getByRole('button', { name: 'Fail T-3' }));
    expect(within(drawer).getByRole('textbox', { name: 'Note for T-3' })).toBeTruthy();
    expect(within(drawer).getByRole('button', { name: 'Send fail' })).toBeTruthy();
  });

  it('attaching a picture with a caption uploads it as a proof of that test', async () => {
    const { drawer, calls } = await openRich();
    const evidence = section(drawer, 'Evidence');
    fireEvent.change(within(evidence).getByRole('textbox', { name: 'Caption for T-1' }), { target: { value: 'the altar again' } });
    const input = evidence.querySelector('input[type="file"]') as HTMLInputElement;
    fireEvent.change(input, { target: { files: [new File(['x'], 'shot.png', { type: 'image/png' })] } });
    await waitFor(() => expect(calls).toHaveLength(1));
    const upload = new URL(calls[0]?.url ?? '', 'http://dashboard');
    expect(upload.pathname).toBe('/api/p/rotp/tests/T-1/proofs');
    expect(upload.searchParams.get('caption')).toBe('the altar again');
    expect(calls[0]?.method).toBe('POST');
  });

  it('a picture with no caption takes its file name as the caption', async () => {
    const { drawer, calls } = await openRich();
    const input = section(drawer, 'Evidence').querySelector('input[type="file"]') as HTMLInputElement;
    fireEvent.change(input, { target: { files: [new File(['x'], 'altar view.png', { type: 'image/png' })] } });
    await waitFor(() => expect(calls).toHaveLength(1));
    expect(new URL(calls[0]?.url ?? '', 'http://dashboard').searchParams.get('caption')).toBe('altar view');
  });
});
