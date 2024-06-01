import type {
  ActionFunctionArgs,
  LoaderFunctionArgs,
  MetaFunction,
} from '@remix-run/node';
import { Form, useLoaderData } from '@remix-run/react';

import api from '../adapters/api';
import Board from '../components/board';
import { Button } from '../components/ui/button';
import { useEffect, useState } from 'react';
import { Chess } from 'chess.js';

export const meta: MetaFunction = () => {
  return [
    { title: 'chesu' },
    {
      name: 'description',
      content: 'A platform to play chess',
    },
  ];
};

export const loader = async ({ request }: LoaderFunctionArgs) => {
  const { data, error } = await (await api.authorized(request)).GET('/user/me');
  const { data: gameData } = await api.unauthorized.GET(
    '/game/52b9eebc-205f-11ef-bf88-570a9abefbb5'
  );

  if (error) {
    return {};
  }

  return {
    user: {
      id: data.user.id,
      username: data.user.username,
    },
    game: {
      ...gameData.game,
    },
  };
};

export const action = async ({ request }: ActionFunctionArgs) => {
  const { data, error } = await (
    await api.authorized(request)
  ).POST('/game/52b9eebc-205f-11ef-bf88-570a9abefbb5', {
    body: {},
  });

  if (error || !data) {
    return {
      error,
    };
  }

  return {};
};

const Game = () => {
  const data = useLoaderData<typeof loader>();
  const [game, setGame] = useState<Record<string, unknown>>(data?.game);

  const getClientColor = () => {
    if (game?.black_player === data?.user?.id) {
      return 'black';
    }

    return 'white';
  };

  const getName = () => {
    if (!data?.user) {
      return 'Opponent';
    }

    return data?.user?.username;
  };

  const getOpponentName = (): string => {
    const clientColor = getClientColor();

    if (clientColor === 'white') {
      if (!game?.black_player) {
        return 'Opponent';
      }

      return game?.black_player as string;
    }

    return game?.white_player as string;
  };

  const renderJoinButton = () => {
    if (game?.white_player === data?.user?.id) {
      return null;
    }

    if (game?.white_player && game?.black_player) {
      return null;
    }

    return (
      <Form method="post">
        <Button className="w-full">Join in the game</Button>
      </Form>
    );
  };

  useEffect(() => {
    const intervalId = setInterval(async () => {
      const response = await fetch(
        `http://localhost:3000/game/52b9eebc-205f-11ef-bf88-570a9abefbb5`
      );

      const json = await response.json();

      console.log('pooling');

      setGame(json.game);
    }, 1000);

    return () => clearInterval(intervalId);
  }, []);

  const handleMove = async (move: string) => {
    const body = JSON.stringify({ game: { new_move: move } });

    const response = await fetch(
      `http://localhost:3000/game/new_move/52b9eebc-205f-11ef-bf88-570a9abefbb5`,
      {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body,
      }
    );

    const json = await response.json();

    setGame(json.game);
  };

  const sanArrayToFen = (sanArray: string[]) => {
    const chess = new Chess();

    for (const move of sanArray) {
      const result = chess.move(move);

      if (result === null) {
        return undefined;
      }
    }

    return chess.fen();
  };

  const fen = sanArrayToFen(game.moves);

  console.log(fen);

  return (
    <div className="h-screen flex items-center justify-center gap-2">
      <div className="flex flex-col gap-4 w-full max-w-[750px] px-6">
        <div className="flex bg-slate-300 p-4 gap-3">
          <div className="bg-slate-800 h-[50px] w-[50px]"></div>
          <span>{getOpponentName()}</span>
        </div>
        <Board
          fen={fen}
          boardOrientation={getClientColor()}
          onMove={handleMove}
        />
        <div className="flex bg-slate-300 p-4 gap-3">
          <div className="bg-slate-800 h-[50px] w-[50px]"></div>
          <span>{getName()}</span>
        </div>
        {renderJoinButton()}
      </div>
    </div>
  );
};

export default Game;
