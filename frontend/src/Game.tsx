import {
  type Component,
  createSignal,
  For,
  onCleanup,
  onMount,
} from 'solid-js';
import { GameV1Api } from './api/game';
import { css } from '../styled-system/css';

const Game: Component<{
  api: GameV1Api;
}> = (props) => {
  const [messages, setMessages] = createSignal<string[]>([]);

  const subscribe = props.api.enterGame().subscribe((event) => {
    setMessages((prev) => [...prev, JSON.stringify(event)]);
  });
  onCleanup(() => subscribe.unsubscribe());

  return (
    <div>
      <h2>游戏</h2>
      <div>
        <For each={messages()}>
          {(message) => (
            <div
              class={css({ padding: '0.5rem', borderBottom: '1px solid #ccc' })}
            >
              {message}
            </div>
          )}
        </For>
      </div>
      <button
        class={css({
          padding: '0.5rem 1rem',
          backgroundColor: '#28a745',
          color: '#fff',
          border: 'none',
          borderRadius: '4px',
          cursor: 'pointer',
          _hover: {
            backgroundColor: '#218838',
          },
        })}
        onClick={async () => {
          await props.api.ping();
        }}
      >
        发送消息
      </button>
    </div>
  );
};

export default Game;
