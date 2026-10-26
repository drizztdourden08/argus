/* @layer renderer-app @kind component */
import { MarkdownEditor } from '@drizztdourden08/tessera';
import { useInRouterContext } from 'react-router';
import { standaloneOptions } from '../behavior/standalone-options';
import type { BoardMarkdownEditorProps } from '../markdown.type';
import { RoutedMarkdownEditor } from './RoutedMarkdownEditor';

const BoardMarkdownEditor = (props: BoardMarkdownEditorProps) =>
  useInRouterContext() ? <RoutedMarkdownEditor {...props} /> : <MarkdownEditor {...props} {...standaloneOptions} />;

export { BoardMarkdownEditor };
