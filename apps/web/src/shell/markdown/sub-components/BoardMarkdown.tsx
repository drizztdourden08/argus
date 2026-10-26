/* @layer renderer-app @kind component */
import { useInRouterContext } from 'react-router';
import { standaloneOptions } from '../behavior/standalone-options';
import type { BoardMarkdownProps } from '../markdown.type';
import { LightboxMarkdown } from './LightboxMarkdown';
import { RoutedMarkdown } from './RoutedMarkdown';

const BoardMarkdown = (props: BoardMarkdownProps) =>
  useInRouterContext() ? <RoutedMarkdown {...props} /> : <LightboxMarkdown {...props} options={standaloneOptions} />;

export { BoardMarkdown };
