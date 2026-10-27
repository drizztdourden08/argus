/* @layer renderer-app @kind test */
import { toast } from '@drizztdourden08/tessera';
import { act, cleanup } from '@testing-library/react';
import { afterEach, vi } from 'vitest';

const cleanAfterEach = (): void => {
  afterEach(() => {
    act(() => toast.clear());
    cleanup();
    vi.unstubAllGlobals();
    localStorage.clear();
  });
};

const cardOf = (container: HTMLElement, tag: string): HTMLElement => container.querySelector<HTMLElement>(`[data-bundle="${tag}"]`) as HTMLElement;

const toggleOf = (card: HTMLElement): HTMLElement => card.querySelector<HTMLElement>('button[aria-expanded]') as HTMLElement;

const isOpen = (card: HTMLElement): boolean => toggleOf(card).getAttribute('aria-expanded') === 'true';

const tagsOf = (container: HTMLElement): (string | null)[] => [...container.querySelectorAll('[data-bundle]')].map((card) => card.getAttribute('data-bundle'));

export { cardOf, cleanAfterEach, isOpen, tagsOf, toggleOf };
