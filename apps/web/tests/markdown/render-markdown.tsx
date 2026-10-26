/* @layer renderer-app @kind test */
import { Markdown } from '@drizztdourden08/tessera';
import { render } from '@testing-library/react';

const renderMarkdown = (source: string): HTMLElement => render(<Markdown>{source}</Markdown>).container;

const texts = (root: ParentNode, selector: string): string[] => [...root.querySelectorAll(selector)].map((element) => element.textContent);

const codeLines = (root: ParentNode): string[] => texts(root, 'pre code > div');

export { codeLines, renderMarkdown, texts };
