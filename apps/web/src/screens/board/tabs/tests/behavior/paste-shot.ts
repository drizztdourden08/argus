/* @layer renderer-app @kind logic */
import type { ClipboardEvent } from 'react';

const pasteShot = (onShot: (file: File) => void) => (event: ClipboardEvent<HTMLElement>): void => {
  const file = Array.from(event.clipboardData.files)[0];
  if (!file) return;
  event.preventDefault();
  onShot(file);
};

export { pasteShot };
