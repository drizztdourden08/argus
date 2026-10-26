/* @layer renderer-app @kind component */
import { MarkdownEditor } from '@drizztdourden08/tessera';
import { useRoutedOptions } from '../behavior/useRoutedOptions';
import type { BoardMarkdownEditorProps } from '../markdown.type';

const RoutedMarkdownEditor = (props: BoardMarkdownEditorProps) => <MarkdownEditor {...props} {...useRoutedOptions()} />;

export { RoutedMarkdownEditor };
