/* @layer renderer-app @kind test */
import { fireEvent, screen } from '@testing-library/react';
import { buildDoc, testDoc } from './fixtures';
import type { Call, Responder } from './mount-tab';

const board = () => ({ builds: [buildDoc('bundle-1')], tests: [testDoc('T-1'), testDoc('T-2')] });

const resultCalls = (calls: Call[]): Call[] => calls.filter((call) => call.url.endsWith('/result'));

const noteBox = (id = 'T-1'): HTMLTextAreaElement => screen.getByRole<HTMLTextAreaElement>('textbox', { name: `Note for ${id}` });

const openBox = async (outcome: string, id = 'T-1'): Promise<HTMLTextAreaElement> => {
  fireEvent.click(await screen.findByRole('button', { name: `${outcome} ${id}` }));
  return noteBox(id);
};

const type = (box: HTMLTextAreaElement, value: string): void => {
  fireEvent.change(box, { target: { value } });
};

const send = (box: HTMLTextAreaElement): void => {
  fireEvent.keyDown(box, { key: 'Enter', ctrlKey: true });
};

const sendButton = (outcome: string): HTMLButtonElement => screen.getByRole<HTMLButtonElement>('button', { name: `Send ${outcome}` });

const refuseResults: Responder = (call) =>
  (call.url.endsWith('/result') ? { status: 200, body: { id: 'O-1', status: 'refused', outcome: 'not your test', entry: null, seq: 1 } } : undefined);

const png = (name = 'shot.png'): File => new File(['x'], name, { type: 'image/png' });

const attach = (file: File): void => {
  const input = document.body.querySelector<HTMLInputElement>('input[type="file"]') as HTMLInputElement;
  fireEvent.change(input, { target: { files: [file] } });
};

export { attach, board, noteBox, openBox, png, refuseResults, resultCalls, send, sendButton, type };
