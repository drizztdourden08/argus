/* @layer renderer-app @kind test */
import { render } from '@testing-library/react';
import { useEffect } from 'react';
import type { ReactNode } from 'react';
import { MemoryRouter, Route, Routes, useLocation } from 'react-router';
import { BoardMarkdown } from '../../src/shell';

const Where = () => {
  const { pathname, search, hash } = useLocation();
  useEffect(() => {
    document.title = `${pathname}${search}${hash}`;
  }, [pathname, search, hash]);
  return null;
};

const inBoard = (children: ReactNode): ReactNode => (
  <MemoryRouter initialEntries={['/p/demo']}>
    <Routes>
      <Route path="/p/:key" element={<>{children}<Where /></>} />
    </Routes>
  </MemoryRouter>
);

const renderMarkdown = (source: string): HTMLElement => render(inBoard(<BoardMarkdown>{source}</BoardMarkdown>)).container;

const texts = (root: ParentNode, selector: string): string[] => [...root.querySelectorAll(selector)].map((element) => element.textContent);

const codeLines = (root: ParentNode): string[] => texts(root, 'pre code > div');

const location = (): string => document.title;

export { codeLines, inBoard, location, renderMarkdown, texts };
