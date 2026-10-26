/* @layer renderer-app @kind component */
import { Markdown } from '@drizztdourden08/tessera';
import { useRoutedOptions } from '../behavior/useRoutedOptions';
import type { BoardMarkdownProps } from '../markdown.type';

const RoutedMarkdown = (props: BoardMarkdownProps) => <Markdown {...props} {...useRoutedOptions()} />;

export { RoutedMarkdown };
