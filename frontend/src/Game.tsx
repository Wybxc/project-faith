import {
  type Component,
  createMemo,
  createSignal,
  For,
  JSXElement,
  Match,
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
  RequestUserEvent,
  UserEvent,
} from './generated/proto/game.v1';
import cards from './cards';

const Game: Component<{
  api: GameV1Api;
}> = (props) => {
  const [starting, setStarting] = createSignal(true);
  const [state, setState] = createStore<GameState>({
    debugLog: [],
    selfHand: [],
    otherHandCount: 0,
    selfDeckCount: 0,
    otherDeckCount: 0,
    roundNumber: 0,
    isMyTurn: false,
    gameFinished: false,
    selfFaith: [],
    otherFaith: [],
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

const Td: Component<{ children?: JSXElement }> = (props) => {
  return (
    <td class={css({ padding: '0.5rem', border: '1px solid #ccc' })}>
      {props.children}
    </td>
  );
};

const GameBoard: Component<{
  state: GameState;
  userEvent: RequestUserEvent | null;
  onFinishEvent: (event?: UserEvent['eventType']) => void;
}> = (props) => {
  return (
    <>
      <table
        class={css({
          width: '100%',
          borderCollapse: 'collapse',
          marginBottom: '1rem',
        })}
      >
        <tbody>
          <tr>
            <Td>当前回合</Td>
            <Td>{props.state.roundNumber}</Td>
          </tr>
          <tr>
            <Td>当前玩家</Td>
            <Td>{props.state.isMyTurn ? '你' : '对方'}</Td>
          </tr>
          <tr>
            <Td>对方牌库剩余</Td>
            <Td>{props.state.otherDeckCount}</Td>
          </tr>
          <tr>
            <Td>对方手牌数</Td>
            <Td>{props.state.otherHandCount}</Td>
          </tr>
          <tr>
            <Td>对方信念牌</Td>
            <Td>
              <For each={props.state.otherFaith}>
                {(card) => <Card cardId={card.cardId} entity={card.entity} />}
              </For>
            </Td>
          </tr>
          <tr>
            <Td>你的牌库剩余</Td>
            <Td>{props.state.selfDeckCount}</Td>
          </tr>
          <tr>
            <Td>你的手牌</Td>
            <Td>
              <For each={props.state.selfHand}>
                {(card) => <Card cardId={card.cardId} entity={card.entity} />}
              </For>
            </Td>
          </tr>
          <tr>
            <Td>你的信念牌</Td>
            <Td>
              <For each={props.state.selfFaith}>
                {(card) => <Card cardId={card.cardId} entity={card.entity} />}
              </For>
            </Td>
          </tr>
        </tbody>
      </table>

      <Show when={props.userEvent} keyed>
        {(userEvent) => (
          <EventInput
            state={props.state}
            userEvent={userEvent}
            onSubmit={props.onFinishEvent}
          />
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
  cardId: number;
  entity: number;
}> = (props) => {
  const card = createMemo(() => cards[props.cardId]);
  return (
    <p title={card()?.description ?? '未知牌'}>
      {props.entity}：{card()?.name ?? '未知牌'}
    </p>
  );
};

let timer: number | null = null;
let lastTime: number | null = null;
function createSingletonTimer(callback: (elapsed: number) => void) {
  if (timer !== null) {
    clearInterval(timer);
    lastTime = null; // Reset lastTime when timer is recreated
  }
  timer = setInterval(() => {
    callback(lastTime === null ? 100 : Date.now() - lastTime);
    lastTime = Date.now();
  }, 100);
}

const EventInput: Component<{
  state: GameState;
  userEvent: RequestUserEvent;
  onSubmit: (event?: UserEvent['eventType']) => void;
}> = (props) => {
  const [time, setTime] = createSignal<number | null>(null);

  createSingletonTimer((elapsed) => {
    setTime((prev) => {
      if (prev === null) return null; // If not enabled, do nothing
      prev -= elapsed / 1000;
      if (prev <= 0) {
        props.onSubmit();
      }
      return prev;
    });
  });
  onMount(() => setTime(props.userEvent.timeout / 1000));
  onCleanup(() => setTime(null));

  return (
    <div>
      <p>剩余时间: {Math.floor(time() ?? 0)} 秒</p>
      <Switch>
        <Match when={props.userEvent.eventType?.$case === 'turnAction'}>
          <TurnActionComponent
            playableCards={props.userEvent.eventType!.value.playableCards}
            onSubmit={props.onSubmit}
          />
        </Match>
      </Switch>
    </div>
  );
};

const TurnActionComponent: Component<{
  playableCards: readonly number[];
  onSubmit: (event: UserEvent['eventType']) => void;
}> = (props) => {
  const [card, setCard] = createSignal<number | null>(null);

  return (
    <div>
      <p>请选择要打出的卡牌:</p>
      <For each={props.playableCards}>
        {(entity) => (
          <label class={css({ display: 'block', margin: '0.5rem 0' })}>
            <input
              type="radio"
              name="playable-card"
              value={entity}
              checked={card() === entity}
              onChange={(e) => setCard(parseInt(e.currentTarget.value, 10))}
              class={css({ marginRight: '0.5rem' })}
            />
            <span>{entity}</span>
          </label>
        )}
      </For>
      <button
        class={css({
          padding: '0.5rem 1rem',
          backgroundColor: '#007bff',
          color: '#fff',
          border: 'none',
          borderRadius: '4px',
          cursor: 'pointer',
        })}
        onClick={() => {
          const entity = card();
          if (entity !== null) {
            props.onSubmit({
              $case: 'playCard',
              value: { entity },
            });
          }
        }}
      >
        Play Card
      </button>

      <button
        class={css({
          padding: '0.5rem 1rem',
          backgroundColor: '#6c757d',
          color: '#fff',
          border: 'none',
          borderRadius: '4px',
          cursor: 'pointer',
        })}
        onClick={() => props.onSubmit({ $case: 'endTurn', value: {} })}
      >
        End Turn
      </button>
    </div>
  );
};

export default Game;
