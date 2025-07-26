import {
  type Component,
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
    <Show
      when={!starting()}
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
    </Show>
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
              {i()}: {card}
            </p>
          )}
        </For>
      </div>

      <Show when={props.userEvent} keyed>
        {(userEvent) => (
          <EventInput
            userEvent={userEvent}
            onSubmit={(event) => {
              if (event === 'timeout') {
                props.onFinishEvent();
              } else {
                props.onFinishEvent(event);
              }
            }}
          />
        )}
      </Show>
    </>
  );
};

const EventInput: Component<{
  userEvent: RequestUserEvent;
  onSubmit: (event: UserEvent['eventType'] | 'timeout') => void;
}> = (props) => {
  const [time, setTime] = createSignal(0);
  const [intervalId, setIntervalId] = createSignal<number | null>(null);

  onMount(() => {
    setTime(20);
    if (intervalId() !== null) {
      clearInterval(intervalId()!);
    }
    const interval = setInterval(() => {
      setTime((prev) => {
        prev -= 1;
        if (prev <= 0) {
          clearInterval(interval);
          setIntervalId(null);
          props.onSubmit('timeout');
        }
        return prev;
      });
    }, 1000);
    setIntervalId(interval);
  });

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
