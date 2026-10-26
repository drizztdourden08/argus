/* @layer renderer-app @kind test */
import { cleanup } from '@testing-library/react';
import { afterEach, vi } from 'vitest';

const cleanAfterEach = (): void => {
  afterEach(() => {
    cleanup();
    vi.unstubAllGlobals();
  });
};

const cardOf = (container: HTMLElement, tag: string): HTMLElement => container.querySelector<HTMLElement>(`[data-bundle="${tag}"]`) as HTMLElement;

const detailsOf = (card: HTMLElement): HTMLDetailsElement => card.querySelector<HTMLDetailsElement>('details') as HTMLDetailsElement;

const tagsOf = (container: HTMLElement): (string | null)[] => [...container.querySelectorAll('[data-bundle]')].map((card) => card.getAttribute('data-bundle'));

export { cardOf, cleanAfterEach, detailsOf, tagsOf };
