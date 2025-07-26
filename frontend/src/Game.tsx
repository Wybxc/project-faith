import { type Component, createSignal, For, onCleanup, Show } from 'solid-js';
import { createStore, reconcile } from 'solid-js/store';
import { GameV1Api } from './api/game';
import { css } from '../styled-system/css';
import { GameState } from './generated/proto/game.v1';

const Game: Component<{
  api: GameV1Api;
}> = (props) => {
  const [waiting, setWaiting] = createSignal(true);
  const [state, setState] = createStore<GameState>({
    selfHand: [],
    otherHandCount: 0,
    selfDeckCount: 0,
    otherDeckCount: 0,
    roundNumber: 0,
  });

  const subscribe = props.api.enterGame().subscribe((event) => {
    switch (event?.$case) {
      case 'stateUpdate':
        setState(reconcile(event.value));
        setWaiting(false);
        break;
      case 'requestUserEvent':
        // Handle user event request
        break;
    }
  });
  onCleanup(() => subscribe.unsubscribe());

  return (
    <Show
      when={!waiting()}
      fallback={
        <div>
          <p class={css({ textAlign: 'center', padding: '20px' })}>
            Waiting for game to start...
          </p>
          <p class={css({ textAlign: 'center', padding: '20px' })}>
            You are in room: {props.api.getRoomName()}
          </p>
        </div>
      }
    >
      <GameBoard state={state} />
    </Show>
  );
};

const GameBoard: Component<{
  state: GameState;
}> = (props) => {
  return (
    <>
      <p>当前回合: {props.state.roundNumber}</p>
      <p>对方牌库剩余: {props.state.otherDeckCount}</p>
      <p>你的牌库剩余: {props.state.selfDeckCount}</p>
      <p>对方手牌数: {props.state.otherHandCount}</p>
      <div>
        你的手牌
        <For each={props.state.selfHand}>{(card) => <p>{card}</p>}</For>
      </div>
    </>
  );
};

export default Game;
