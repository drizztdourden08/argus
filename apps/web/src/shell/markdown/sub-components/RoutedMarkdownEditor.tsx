/* @layer renderer-app @kind component */
import { useRoutedOptions } from '../behavior/useRoutedOptions';
import type { BoardMarkdownEditorProps } from '../markdown.type';
import { StandaloneEditor } from './StandaloneEditor';

const RoutedMarkdownEditor = (props: BoardMarkdownEditorProps) => <StandaloneEditor {...props} options={useRoutedOptions()} />;

export { RoutedMarkdownEditor };
