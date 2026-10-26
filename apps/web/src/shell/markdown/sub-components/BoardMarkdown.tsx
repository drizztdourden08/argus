/* @layer renderer-app @kind component */
import { Markdown } from '@drizztdourden08/tessera';
import { useInRouterContext } from 'react-router';
import { standaloneOptions } from '../behavior/standalone-options';
import type { BoardMarkdownProps } from '../markdown.type';
import { RoutedMarkdown } from './RoutedMarkdown';

const BoardMarkdown = (props: BoardMarkdownProps) =>
  useInRouterContext() ? <RoutedMarkdown {...props} /> : <Markdown {...props} {...standaloneOptions} />;

export { BoardMarkdown };
