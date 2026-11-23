/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import type { StreamInner, TranscriptLine } from '../../src/data/overlord/overlord-transcript.type';
import { liveDraft } from '../../src/shell/OverlordChat/behavior/live-draft';
import { tailOf } from '../../src/shell/OverlordChat/behavior/tail-of';
import { toRows } from '../../src/shell/OverlordChat/behavior/to-rows';
import { workingActivity } from '../../src/shell/OverlordChat/behavior/working-activity';

const assistant = (index: number, content: unknown[], argusAt?: string): TranscriptLine => ({ index, event: { type: 'assistant', argusAt, message: { content } } });
const stream = (index: number, inner: StreamInner): TranscriptLine => ({ index, event: { type: 'stream_event', event: inner } });
const start = (index: number, at: number, type: string, name?: string): TranscriptLine => stream(index, { type: 'content_block_start', index: at, content_block: { type, name } });
const delta = (index: number, at: number, body: Record<string, string>): TranscriptLine => stream(index, { type: 'content_block_delta', index: at, delta: body });

describe('L1 live thinking and text', { tags: ['critical'] }, () => {
  it('grows a thinking row and a text row from the deltas after the last whole event', () => {
    const draft = liveDraft([
      assistant(1, [{ type: 'text', text: 'Old.' }]),
      start(2, 0, 'thinking'),
      delta(3, 0, { type: 'thinking_delta', thinking: 'Let me ' }),
      delta(4, 0, { type: 'thinking_delta', thinking: 'check.' }),
      start(5, 1, 'text'),
      delta(6, 1, { type: 'text_delta', text: 'Hello' }),
    ]);
    expect(draft.rows).toMatchObject([
      { kind: 'thinking', text: 'Let me check.', live: true },
      { kind: 'assistant', text: 'Hello', live: true },
    ]);
    expect(draft.tool).toBeNull();
  });

  it('drops the draft once the whole event arrives and names a streaming tool', () => {
    const done = liveDraft([start(1, 0, 'text'), delta(2, 0, { type: 'text_delta', text: 'Hi' }), assistant(3, [{ type: 'text', text: 'Hi' }])]);
    expect(done.rows).toHaveLength(0);
    expect(liveDraft([start(1, 0, 'tool_use', 'Bash')]).tool).toBe('Bash');
  });
});

describe('L2 working line and times', { tags: ['critical'] }, () => {
  it('says what the Overlord does now', () => {
    const none = { rows: [], tool: null };
    const running = toRows([assistant(1, [{ type: 'tool_use', name: 'Bash', id: 't1', input: { command: 'ls' } }])]);
    expect(workingActivity(running, none)).toEqual({ text: 'Running Bash: ls', mark: 'working' });
    expect(workingActivity([], none)).toEqual({ text: 'Thinking', mark: 'thinking' });
    expect(workingActivity([], { rows: [], tool: 'Read' })).toEqual({ text: 'Running Read', mark: 'working' });
    expect(workingActivity([], { rows: [{ kind: 'assistant', key: 'k', text: 'x' }], tool: null }).text).toBe('Writing');
  });

  it('keeps the arrival time of a line on its rows and clips a live tail', () => {
    const rows = toRows([assistant(1, [{ type: 'text', text: 'Hi' }], '2026-11-22T10:00:00Z')]);
    expect(rows[0]).toMatchObject({ at: '2026-11-22T10:00:00Z' });
    expect(tailOf('a  b\nc')).toBe('a b c');
    expect(tailOf('x'.repeat(200))).toHaveLength(93);
  });
});
