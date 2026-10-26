/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { mergeRunLines } from '../../src/data/runs/merge-run-lines';
import type { RunEntry, RunLogLine } from '../../src/data/runs';
import {
  countsText, launcherViews, logRows, noPlayHint, runDuration, runNote, terminalLast, terminalMeta,
} from '../../src/screens/board/tabs/builds/model';
import { formatDuration } from '../../src/screens/board/tabs/builds/model/format-duration';
import { DAY, launcherDoc, runDoc, runningDoc, standaloneDoc } from './fixtures';
import { at, input, launchers } from './model-docs';

describe('texts of a launcher and a run', () => {
  it('counts the launchers to start, stale and retired', () => {
    const none = launcherViews(input({ launchers: launchers() }));
    expect(countsText(none)).toBe('');
    const views = launcherViews(input({
      launchers: launchers(launcherDoc('master'), standaloneDoc('s', { lastUsedAt: at(30 * DAY) }), launcherDoc('r', { retiredAt: at(DAY) })),
    }));
    expect(countsText(views)).toBe('2 to start · 1 stale · 1 retired');
    expect(countsText(launcherViews(input()))).toBe('1 to start');
  });

  it('says why Play is missing', () => {
    expect(noPlayHint('master')).toBe('No Play for launcher master: its command spans several lines, or its run name is taken (coord launcher show master)');
  });

  it('writes the meta and the last line of a terminal', () => {
    const live = runningDoc('R-4', 'launcher-master', { lastLine: 'listening' }) as RunEntry;
    expect(terminalMeta(live)).toBe('R-4 · running · input on');
    expect(terminalLast(live)).toBe('listening');
    expect(terminalMeta({ ...live, stdin: false })).toBe('R-4 · running');
    expect(terminalLast({ ...live, lastLine: null })).toBe('Launching...');
    expect(terminalMeta({ ...live, status: 'queued' })).toBe('R-4 · starting');
    const failed = { ...live, status: 'failed', exitCode: 3, lastLine: null } as RunEntry;
    expect(terminalMeta(failed)).toBe('R-4 · failed (exit 3)');
    expect(terminalLast(failed)).toBe('');
  });

  it('notes a run by its last line, else its error', () => {
    const run = runDoc('R-1', 'launcher-master', { status: 'failed', exitCode: null, error: 'timeout', lastLine: null }) as RunEntry;
    expect(runNote(run)).toBe('timeout');
    expect(runNote({ ...run, lastLine: 'boom' })).toBe('boom');
    expect(runNote({ ...run, error: null })).toBe('');
  });

  it('formats a duration the way a build is read', () => {
    expect([0, 59_000, 65_000, 3_599_000, 3_900_000].map(formatDuration)).toEqual(['0s', '59s', '1m 05s', '59m 59s', '1h 05m']);
  });

  it('measures a finished run from its start, and nothing for a live one', () => {
    const done = runDoc('R-1', 'x', { startedAt: at(125_000), endedAt: at(0) }) as RunEntry;
    expect(runDuration(done)).toBe('2m 05s');
    expect(runDuration(runningDoc('R-2', 'x') as RunEntry)).toBe('');
  });
});

describe('logRows', () => {
  const line = (seq: number, stream: RunLogLine['stream'], text: string): RunLogLine => ({ seq, stream, at: '2026-10-26T10:00:00.000Z', text });

  it('keys a row by its sequence and keeps the stream as the kind', () => {
    const rows = logRows([line(7, 'out', 'hello\n'), line(8, 'err', 'bad'), line(-1, 'sys', 'view cleared')]);
    expect(rows.map((row) => row.id)).toEqual(['7', '8', '-1']);
    expect(rows.map((row) => row.kind)).toEqual(['out', 'err', 'sys']);
    expect(rows.map((row) => row.tag)).toEqual(['', 'err', 'sys']);
    expect(rows[0]?.message).toBe('hello');
  });

  it('leaves a missing time empty', () => {
    expect(logRows([{ seq: -1, stream: 'sys', at: '', text: 'x' }])[0]?.gutter).toBe('');
  });
});

describe('mergeRunLines', () => {
  const l = (seq: number, text = `line ${seq}`): RunLogLine => ({ seq, stream: 'out', at: '', text });

  it('drops a line already seen by its sequence and keeps the order', () => {
    const merged = mergeRunLines([l(1), l(2)], [l(2), l(3), l(3), l(4)], 300);
    expect(merged.map((line) => line.seq)).toEqual([1, 2, 3, 4]);
  });

  it('always keeps a local line (no sequence)', () => {
    const merged = mergeRunLines([l(5)], [{ seq: -1, stream: 'sys', at: '', text: 'note' }, { seq: -2, stream: 'sys', at: '', text: 'again' }], 300);
    expect(merged.map((line) => line.text)).toEqual(['line 5', 'note', 'again']);
  });

  it('keeps only the newest lines up to the cap', () => {
    const many = Array.from({ length: 400 }, (_, index) => l(index + 1));
    const merged = mergeRunLines([], many, 300);
    expect(merged).toHaveLength(300);
    expect(merged[0]?.text).toBe('line 101');
    expect(merged.at(-1)?.text).toBe('line 400');
  });
});
