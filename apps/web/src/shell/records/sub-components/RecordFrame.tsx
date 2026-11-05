/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { useFrameOpen } from '../behavior/useFrameOpen';
import type { RecordFrameProps } from '../record-frame.type';
import { FrameHead } from './FrameHead';
import './RecordFrame.css';

const RecordFrame = (props: RecordFrameProps) => {
  const { name, kind, icon, selection, code, title, compactTitle, pinTitle, content, menu, density = 'card', attrs, onClickCapture, children } = props;
  const [open, toggle] = useFrameOpen(props);
  const collapsedTitle = open ? null : (compactTitle ?? title);
  const headTitle = pinTitle ? title : collapsedTitle;
  return (
    <Flex direction="column" className={`record-frame record-frame--${density}`} {...attrs} onClickCapture={onClickCapture}>
      <FrameHead open={open} onToggle={toggle} name={name} kind={kind} icon={icon} selection={selection} code={code} headTitle={headTitle} content={content} menu={menu} />
      {open && (
        <Flex direction="column" gap="xs" className="record-frame__body">
          {!pinTitle && title}
          {children}
        </Flex>
      )}
    </Flex>
  );
};

export { RecordFrame };
