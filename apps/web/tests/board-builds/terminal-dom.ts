/* @layer renderer-app @kind test */
import { fireEvent, screen } from '@testing-library/react';
import type { Call, Responder } from '../board-tests/mount-tab';
import { defDoc, launcherDoc, runningDoc } from './fixtures';

const line = (seq: number, text: string, stream = 'out') => ({ seq, stream, at: '2026-10-26T10:00:00.000Z', text });

const tailOf = (lines: object[], next = lines.length, dropped = 0): Responder => (call) =>
  (call.url.includes('/logs?') ? { status: 200, body: { lines, next, dropped } } : undefined);

const board = (run: object = runningDoc('R-4', 'launcher-master', { lastLine: 'compiling main.ts' })) => ({
  launchers: [launcherDoc('master')],
  launches: [defDoc('master')],
  runs: [run as Record<string, unknown>],
});

const summaryOf = (): HTMLElement => screen.getByText('Terminal').closest('summary') as HTMLElement;

const waitForSummary = async (): Promise<HTMLElement> => {
  await screen.findByText('Terminal');
  return summaryOf();
};

const openTerminal = (): void => {
  fireEvent.click(summaryOf());
};

const sendLine = (input: HTMLElement, text: string): void => {
  fireEvent.change(input, { target: { value: text } });
  fireEvent.keyDown(input, { key: 'Enter' });
};

const typeIntoTerminal = async (text: string): Promise<HTMLInputElement> => {
  await waitForSummary();
  openTerminal();
  const input = await screen.findByLabelText<HTMLInputElement>('Send a line to the process');
  sendLine(input, text);
  return input;
};

const isOpen = (): boolean => (summaryOf().closest('details') as HTMLDetailsElement).open;

const logCalls = (calls: Call[]): Call[] => calls.filter((call) => call.url.includes('/logs'));

const stdinCalls = (calls: Call[]): Call[] => calls.filter((call) => call.url.endsWith('/stdin'));

export { board, isOpen, sendLine, typeIntoTerminal, line, logCalls, openTerminal, stdinCalls, summaryOf, tailOf, waitForSummary };
