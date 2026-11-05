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

const cardOf = (container: HTMLElement, id: string): HTMLElement => container.querySelector<HTMLElement>(`[data-run="${id}"]`) as HTMLElement;

const toggleOf = (card: HTMLElement): HTMLElement => card.querySelector<HTMLElement>('summary') as HTMLElement;

const isOpen = (card: HTMLElement): boolean => (card.querySelector('details') as HTMLDetailsElement).open;

const tagsOf = (container: HTMLElement): (string | null)[] => [...container.querySelectorAll('[data-run]')].map((card) => card.getAttribute('data-run'));

export { cardOf, cleanAfterEach, isOpen, tagsOf, toggleOf };
