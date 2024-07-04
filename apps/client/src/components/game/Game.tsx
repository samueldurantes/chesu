import { useEffect, useRef, useState } from 'react';
import { useParams } from 'react-router-dom';
import {
  useSuspenseQueries,
  useMutation,
  useQueryClient,
} from '@tanstack/react-query';

import api from '../../api/api';
import { Board } from '../ui/Board';
import { Card } from '../ui/Card';
import { Button } from '../ui/Button';

const Game = () => {
  const connection = useRef<WebSocket | null>(null);
  const params = useParams();

  const [{ data: queryUser }, { data: queryGame }] = useSuspenseQueries({
    queries: [
      {
        queryKey: ['user/me'],
        queryFn: () => api.GET('/user/me'),
        networkMode: 'always',
      },
      {
        queryKey: ['game/detail'],
        queryFn: async () => {
          const { data, error } = await api.GET('/game/{id}', {
            params: {
              path: {
                id: params.id as string,
              },
            },
          });

          if (error) {
            throw new Error(error.message);
          }

          return data;
        },
      },
    ],
  });

  const getColorLoggedPlayer = () => {
    if (queryGame?.game?.white_player?.id === queryUser?.data?.user?.id) {
      return 'white';
    }

    return 'black';
  };

  const colorLoggedPlayer = getColorLoggedPlayer();

  const [opponent, setOpponent] = useState<string>(() => {
    const whitePlayer = queryGame?.game?.white_player;
    const blackPlayer = queryGame?.game?.black_player;

    if (colorLoggedPlayer === 'white') {
      if (!blackPlayer) {
        return 'Waiting for opponent...';
      }

      return blackPlayer.username;
    }

    if (!whitePlayer) {
      return 'Waiting for opponent...';
    }

    return whitePlayer.username;
  });

  const [san, setSan] = useState<string[]>(() => {
    if (queryGame?.game?.id !== (params.id as string)) {
      return [];
    }

    return queryGame?.game?.moves || [];
  });

  useEffect(() => {
    // TODO: Move this to .env file
    const socket = new WebSocket('ws://localhost:3000/game/ws');

    socket.addEventListener('open', () => {
      socket.send(params.id as string);
    });

    socket.addEventListener('message', (event) => {
      const receiverData = JSON.parse(event.data);

      if (receiverData?.event === 'join') {
        if (receiverData?.data?.id !== queryUser.data?.user?.id) {
          setOpponent(receiverData?.data?.username);
        }

        return;
      }

      if (receiverData?.message) {
        return;
      }

      setSan((prevSan) => [...prevSan, receiverData.play_move]);
    });

    connection.current = socket;

    return () => {
      socket.close();
    };
  }, [params, queryUser]);

  const queryClient = useQueryClient();

  const { mutateAsync, isPending } = useMutation({
    mutationFn: async () => {
      const { data, error } = await api.POST('/game/{id}', {
        params: {
          path: {
            id: params.id as string,
          },
        },
      });

      if (error) {
        throw new Error(error.message);
      }

      return data;
    },
    onSuccess: (data) => {
      queryClient.setQueryData(['game/detail'], data);
    },
    onError: (error) => console.log({ error }),
  });

  const getJoinButton = () => {
    const whitePlayer = queryGame?.game?.white_player;
    const blackPlayer = queryGame?.game?.black_player;

    if (whitePlayer?.id === queryUser?.data?.user?.id) {
      return null;
    }

    if (blackPlayer?.id === queryUser?.data?.user?.id) {
      return null;
    }

    return (
      <Button onClick={() => mutateAsync()} disabled={isPending}>
        Join
      </Button>
    );
  };

  return (
    <div className="h-screen flex items-center justify-center gap-2 bg-[#121212]">
      <div className="flex flex-col gap-4 w-full max-w-[750px] px-6">
        <Card className="flex items-center justify-between p-4 bg-[#1e1e1e]">
          <div className="flex gap-2 items-center">
            <div className="bg-white h-[50px] w-[50px]"></div>
            <span className="text-white">{opponent}</span>
          </div>
        </Card>

        <Board
          boardOrientation={colorLoggedPlayer}
          san={san}
          onMove={(move) => {
            connection.current?.send(
              JSON.stringify({
                game_id: params.id,
                player_id: queryUser?.data?.user?.id,
                play_move: move,
              })
            );
          }}
        />

        <Card className="flex items-center justify-between p-4 bg-[#1e1e1e]">
          <div className="flex gap-2 items-center">
            <div className="bg-white h-[50px] w-[50px]"></div>
            <span className="text-white">
              {queryUser?.data?.user?.username}
            </span>
          </div>
          {getJoinButton()}
        </Card>
      </div>
    </div>
  );
};

export default Game;
