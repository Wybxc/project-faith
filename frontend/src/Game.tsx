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
    debugLog: [],
    selfHand: [],
    otherHandCount: 0,
    selfDeckCount: 0,
    otherDeckCount: 0,
    roundNumber: 0,
    isMyTurn: false,
    gameFinished: false,
    selfFaithCards: [],
    otherFaithCards: [],
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
              <For each={props.state.otherFaithCards}>
                {(card) => (
                  <span class={css({ margin: '0 0.5rem' })}>
                    <Card cardId={card.toString()} />
                  </span>
                )}
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
                {(card, i) => (
                  <div>
                    {i()}: <Card cardId={card.toString()} />
                  </div>
                )}
              </For>
            </Td>
          </tr>
          <tr>
            <Td>你的信念牌</Td>
            <Td>
              <For each={props.state.selfFaithCards}>
                {(card) => (
                  <span class={css({ margin: '0 0.5rem' })}>
                    <Card cardId={card.toString()} />
                  </span>
                )}
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
  state: GameState;
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
            handCount={props.state.selfHand.length}
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
  handCount: number;
  onSubmit: (event: PlayCard) => void;
}> = (props) => {
  const [cardId, setCardId] = createSignal('');

  return (
    <div>
      <p>请选择要打出的卡牌索引（0 - {props.handCount - 1}）:</p>
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
        onClick={() => {
          const cardIdx = parseInt(cardId(), 10);
          if (cardIdx >= 0 && cardIdx < props.handCount) {
            props.onSubmit({ cardIdx });
          } else {
            alert('无效的卡牌索引');
          }
        }}
      >
        Play Card
      </button>
    </div>
  );
};

export default Game;
