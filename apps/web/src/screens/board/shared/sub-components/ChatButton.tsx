/* @layer renderer-app @kind component */
import { Icon, IconButton } from '@drizztdourden08/tessera';
import type { ChatButtonProps } from '../shared.type';
import './ChatButton.css';

const ChatButton = (props: ChatButtonProps) => {
  const { link, label, size = 'sm' } = props;
  return (
    <IconButton label={label} variant="ghost" size={size} onClick={() => window.location.assign(link)}>
      <Icon name="message-square" className="chat-button__icon" />
    </IconButton>
  );
};

export { ChatButton };
