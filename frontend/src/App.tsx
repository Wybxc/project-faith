import { createSignal, For, onMount, Show, type Component } from 'solid-js';
import { css } from '../styled-system/css';
import { GameV1Api } from './api/game';
import { AuthV1Api } from './api/auth';
import Game from './Game';

const Input: Component<{
  id: string;
  value?: string;
  setValue?: (value: string) => void;
}> = (props) => {
  return (
    <input
      type="text"
      id={props.id}
      class={css({
        padding: '0.5rem',
        border: '1px solid #ccc',
        borderRadius: '4px',
        marginBottom: '1rem',
      })}
      value={props.value || ''}
      onInput={(e) => {
        if (props.setValue) {
          props.setValue(e.target.value);
        }
      }}
    />
  );
};

const FormButton: Component<{
  value?: string;
}> = (props) => {
  return (
    <input
      type="submit"
      value={props.value || '提交'}
      class={css({
        padding: '0.5rem 1rem',
        backgroundColor: '#007bff',
        color: '#fff',
        border: 'none',
        borderRadius: '4px',
        cursor: 'pointer',
        _hover: {
          backgroundColor: '#0056b3',
        },
      })}
    />
  );
};

const Login: Component<{
  setApi: (api: GameV1Api) => void;
}> = (props) => {
  const [username, setUsername] = createSignal('');
  const [roomName, setRoomName] = createSignal('');

  return (
    <form
      class={css({
        display: 'flex',
        flexDirection: 'column',
        width: '300px',
        padding: '1rem',
        border: '1px solid #ccc',
        borderRadius: '8px',
        margin: '2rem auto',
        backgroundColor: '#f9f9f9',
      })}
      onSubmit={async (e) => {
        e.preventDefault();
        const token = await new AuthV1Api().login(username());
        if (!token) {
          alert('登录失败，请检查用户名。');
          return;
        }
        const api = new GameV1Api(token);
        await api.joinRoom(roomName());
        props.setApi(api);
      }}
    >
      <label>用户名：</label>
      <Input id="username" value={username()} setValue={setUsername} />
      <label>房间号：</label>
      <Input id="roomName" value={roomName()} setValue={setRoomName} />
      <FormButton value="加入房间" />
    </form>
  );
};

const App: Component = () => {
  const [api, setApi] = createSignal<GameV1Api | null>(null);

  return (
    <div>
      <Show when={api()} fallback={<Login setApi={setApi} />} keyed>
        {(api) => <Game api={api} />}
      </Show>
    </div>
  );
};

export default App;
