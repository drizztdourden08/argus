/* @layer renderer-app @kind component */
import { useInRouterContext } from 'react-router';
import { standaloneOptions } from '../behavior/standalone-options';
import type { BoardMarkdownEditorProps } from '../markdown.type';
import { RoutedMarkdownEditor } from './RoutedMarkdownEditor';
import { StandaloneEditor } from './StandaloneEditor';

const BoardMarkdownEditor = (props: BoardMarkdownEditorProps) =>
  useInRouterContext() ? <RoutedMarkdownEditor {...props} /> : <StandaloneEditor {...props} options={standaloneOptions} />;

export { BoardMarkdownEditor };
