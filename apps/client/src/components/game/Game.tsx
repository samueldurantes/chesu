import { useEffect, useRef, useState } from 'react';
import { useParams } from 'react-router-dom';
import {
  useSuspenseQueries,
  useQueryClient
} from '@tanstack/react-query';

import api from '../../api/api';
import { Board } from '../ui/Board';
import { Card } from '../ui/Card';

const Game = () => {
  const connection = useRef<WebSocket | null>(null);
  const params = useParams();
  const queryClient = useQueryClient();

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
              path: { id: params.id as string, },
            },
          });

          if (error) throw new Error(error.message);

          return data;
        },
      },
    ],
  });

  const [san, setSan] = useState<string[]>(() => {
    if (queryGame?.game?.id !== (params.id as string)) {
      return [];
    }

    return queryGame?.game?.moves || [];
  });

  useEffect(() => {
    // if (connection.current) return;

    // TODO: Move this to .env file
    const socket = new WebSocket('ws://localhost:3000/game/ws');

    socket.addEventListener('open', () => {
      socket.send(params.id as string);
    });

    socket.addEventListener('message', (event) => {
      const receiverData = JSON.parse(event.data);

      switch (receiverData?.event) {
        case "join":
          queryClient.refetchQueries({ queryKey: ['game/detail'] });
          break;
        case "PlayMove":
          setSan((prevSan) => [...prevSan, receiverData.data.move_played]);
          break;
        default: return;
      }
    });

    connection.current = socket;

    return () => {
      socket.close();
    };
  }, [params, queryUser]);

  const getColorLoggedPlayer = () => {
    if (queryGame?.game?.black_player?.id === queryUser?.data?.user?.id)
      return 'black';

    return 'white';
  };

  const getUsernames = () => {
    return {
      top: getColorLoggedPlayer() !== "white" ? queryGame?.game?.white_player?.username : queryGame?.game?.black_player?.username,
      bottom: getColorLoggedPlayer() === "white" ? queryGame?.game?.white_player?.username : queryGame?.game?.black_player?.username,
    }
  }

  const playMove = (move: string) => {
    connection.current?.send(
      JSON.stringify({
        event: "PlayMove",
        data: {
          game_id: params.id,
          player_id: queryUser?.data?.user?.id,
          move_played: move,
        }
      })
    )
  }

  return (
    <div className="h-screen flex items-center justify-center gap-2 bg-[#121212]">
      <div className="flex flex-col gap-4 w-full max-w-[750px] px-6">
        <Card className="flex items-center justify-between p-4 bg-[#1e1e1e]">
          <div className="flex gap-2 items-center">
            <div className="bg-white h-[50px] w-[50px]"></div>
            <span className="text-white">{getUsernames().top || "Waiting oponent..."}</span>
          </div>
        </Card>

        <Board
          boardOrientation={getColorLoggedPlayer()}
          san={san}
          onMove={playMove}
        />

        <Card className="flex items-center justify-between p-4 bg-[#1e1e1e]">
          <div className="flex gap-2 items-center">
            <div className="bg-white h-[50px] w-[50px]"></div>
            <span className="text-white">
              {getUsernames().bottom}
            </span>
          </div>
        </Card>
      </div>
    </div>
  );
};

export default Game;
