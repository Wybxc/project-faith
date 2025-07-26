import {
  type Component,
  createMemo,
  createSignal,
  For,
  Match,
  on,
  onCleanup,
  onMount,
  Show,
  Switch,
} from 'solid-js';
import { createStore, reconcile } from 'solid-js/store';
import { GameV1Api } from './api/game';
import { css } from '../styled-system/css';
import {
  GameState,
  PlayCard,
  RequestUserEvent,
  UserEvent,
} from './generated/proto/game.v1';
import cards from './assets/cards.json';

const Game: Component<{
  api: GameV1Api;
}> = (props) => {
  const [starting, setStarting] = createSignal(true);
  const [state, setState] = createStore<GameState>({
    selfHand: [],
    otherHandCount: 0,
    selfDeckCount: 0,
    otherDeckCount: 0,
    roundNumber: 0,
    gameFinished: false,
    debugLog: [],
  });
  const [userEvent, setUserEvent] = createSignal<RequestUserEvent | null>(null);

  const subscribe = props.api.enterGame().subscribe((event) => {
    switch (event?.$case) {
      case 'stateUpdate':
        setState(reconcile(event.value));
        setStarting(false);
        break;
      case 'requestUserEvent':
        setUserEvent(event.value);
        break;
    }
  });
  onCleanup(() => subscribe.unsubscribe());

  return (
    <Switch
      fallback={
        <GameBoard
          state={state}
          userEvent={userEvent()}
          onFinishEvent={(event) => {
            const seqnum = userEvent()?.seqnum;
            if (event && seqnum) {
              props.api.submitUserEvent(seqnum, event);
            }
            setUserEvent(null);
          }}
        />
      }
    >
      <Match when={starting()}>
        <div>
          <p class={css({ textAlign: 'center', padding: '20px' })}>
            Waiting for game to start...
          </p>
          <p class={css({ textAlign: 'center', padding: '20px' })}>
            You are in room: {props.api.getRoomName()}
          </p>
        </div>
      </Match>
      <Match when={state.gameFinished}>
        <div class={css({ textAlign: 'center', padding: '20px' })}>
          <p>游戏已结束！</p>
        </div>
      </Match>
    </Switch>
  );
};

const GameBoard: Component<{
  state: GameState;
  userEvent: RequestUserEvent | null;
  onFinishEvent: (event?: UserEvent['eventType']) => void;
}> = (props) => {
  return (
    <>
      <p>当前回合: {props.state.roundNumber}</p>
      <p>对方牌库剩余: {props.state.otherDeckCount}</p>
      <p>你的牌库剩余: {props.state.selfDeckCount}</p>
      <p>对方手牌数: {props.state.otherHandCount}</p>
      <div>
        你的手牌
        <For each={props.state.selfHand}>
          {(card, i) => (
            <p>
              {i()}: <Card cardId={card.toString()} />
            </p>
          )}
        </For>
      </div>

      <Show when={props.userEvent} keyed>
        {(userEvent) => (
          <EventInput userEvent={userEvent} onSubmit={props.onFinishEvent} />
        )}
      </Show>

      <details>
        <summary>调试日志</summary>
        <ul>
          <For each={props.state.debugLog}>{(log) => <li>{log}</li>}</For>
        </ul>
      </details>
    </>
  );
};

const Card: Component<{
  cardId: string;
}> = (props) => {
  const card = createMemo(() => cards[props.cardId as keyof typeof cards]);
  return (
    <span title={card()?.description ?? '未知牌'}>
      {card()?.name ?? '未知牌'}
    </span>
  );
};

let timer: number | null = null;
function createSingletonTimer(callback: () => void) {
  if (timer !== null) {
    clearInterval(timer);
  }
  timer = setInterval(callback, 1000);
}

const EventInput: Component<{
  userEvent: RequestUserEvent;
  onSubmit: (event?: UserEvent['eventType']) => void;
}> = (props) => {
  const [time, setTime] = createSignal<number | null>(null);

  createSingletonTimer(() => {
    setTime((prev) => {
      if (prev === null) return null; // If not enabled, do nothing
      prev -= 1;
      if (prev <= 0) {
        props.onSubmit();
      }
      return prev;
    });
  });
  onMount(() => setTime(props.userEvent.timeout));
  onCleanup(() => setTime(null));

  return (
    <div>
      <p>剩余时间: {time()} 秒</p>
      <Switch>
        <Match when={props.userEvent.eventType?.$case === 'playCard'}>
          <PlayCardComponent
            onSubmit={(event) =>
              props.onSubmit({
                $case: 'playCard',
                value: event,
              })
            }
          />
        </Match>
      </Switch>
    </div>
  );
};

const PlayCardComponent: Component<{
  onSubmit: (event: PlayCard) => void;
}> = (props) => {
  const [cardId, setCardId] = createSignal('');

  return (
    <div>
      <input
        type="text"
        value={cardId()}
        onInput={(e) => setCardId(e.currentTarget.value)}
        class={css({
          padding: '0.5rem',
          border: '1px solid #ccc',
          borderRadius: '4px',
          marginBottom: '1rem',
        })}
      />
      <button
        onClick={() =>
          props.onSubmit({
            cardIdx: parseInt(cardId(), 10),
          })
        }
      >
        Play Card
      </button>
    </div>
  );
};

export default Game;
