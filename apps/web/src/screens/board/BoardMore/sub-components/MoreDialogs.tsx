/* @layer renderer-app @kind component */
import type { MoreDialogsProps } from '../BoardMore.type';
import { AvatarDialog } from './AvatarDialog';
import { LegendDialog } from './LegendDialog';
import { ResourcesDialog } from './ResourcesDialog';

const MoreDialogs = (props: MoreDialogsProps) => {
  const { boardKey, dialog, locks, held, onClose } = props;
  if (dialog === 'locks') return <ResourcesDialog locks={locks} held={held} onClose={onClose} />;
  if (dialog === 'legend') return <LegendDialog onClose={onClose} />;
  if (dialog === 'avatar') return <AvatarDialog boardKey={boardKey} onClose={onClose} />;
  return null;
};

export { MoreDialogs };
