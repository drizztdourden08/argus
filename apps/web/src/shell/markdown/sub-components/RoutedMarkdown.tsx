/* @layer renderer-app @kind component */
import { useRoutedOptions } from '../behavior/useRoutedOptions';
import type { BoardMarkdownProps } from '../markdown.type';
import { LightboxMarkdown } from './LightboxMarkdown';

const RoutedMarkdown = (props: BoardMarkdownProps) => <LightboxMarkdown {...props} options={useRoutedOptions()} />;

export { RoutedMarkdown };
