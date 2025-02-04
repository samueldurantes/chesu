import { useEffect, useRef, useState } from 'react';
import { useParams } from 'react-router-dom';
import {
  useSuspenseQueries,
  useQueryClient
} from '@tanstack/react-query';

import api from '../../api/api';
import { Board } from '../ui/Board';
import Header from '../header/Header';
import GameDisplay from './GameDisplay';
import GameInfo from './GameInfo';

enum PlayerType {
  White,
  Black,
  Viewer
}

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
    return queryGame?.game?.moves || [];
  });

  useEffect(() => {
    if (connection.current) return;

    // TODO: Move this to .env file
    const socket = new WebSocket('ws://localhost:3000/game/ws');

    socket.addEventListener('open', () => {
      socket.send(params.id as string);
    });

    socket.addEventListener('message', (event) => {
      console.table(event.data)
      const message = JSON.parse(event.data);

      switch (message?.event) {
        case "Join":
          queryClient.refetchQueries({ queryKey: ['game/detail'] });
          break;
        case "PlayMove":
          setSan(prevSan => [...prevSan, message.data.move_played]);
          break;
        case "GameChangeState":
          if (message?.data != "Running") {
            alert(message?.data)
            queryClient.refetchQueries({ queryKey: ["user/me"] })
          }
          break;
        default: return;
      }
    });

    window.addEventListener("close", () => disconnect(socket));
    window.addEventListener("popstate", () => disconnect(socket));

    connection.current = socket;

  }, [params, queryUser]);


  const getPlayerType = () => {
    if (queryGame?.game?.white_player?.id === queryUser?.data?.user?.id)
      return PlayerType.White;

    if (queryGame?.game?.black_player?.id === queryUser?.data?.user?.id)
      return PlayerType.Black;

    return PlayerType.Viewer;
  };


  const getUsernames = () => {
    if (getPlayerType() == PlayerType.Black)
      return {
        top: queryGame?.game?.white_player.username,
        bottom: queryGame?.game?.black_player.username
      }

    return {
      top: queryGame?.game?.black_player?.username,
      bottom: queryGame?.game?.white_player?.username
    }
  }

  const disconnect = (socket: WebSocket) => {
    if (socket.readyState !== WebSocket.OPEN) return;

    const data = {
      game_id: params.id,
      player_id: queryUser?.data?.user?.id,
    };

    socket.send(JSON.stringify({ event: "Disconnect", data }))
    socket.close();
  }

  const playMove = (move: string) => {
    const data = {
      game_id: params.id,
      player_id: queryUser?.data?.user?.id,
      move_played: move,
    };

    connection.current?.send(JSON.stringify({ event: "PlayMove", data }))
  }

  const boardOrientation = () => {
    return getPlayerType() == PlayerType.Black ? "black" : "white"
  }

  return (
    <div className="h-full min-h-screen flex flex-col items-center gap-2 bg-[#121212]">
      <Header user={queryUser?.data?.user} />
      <div className="flex flex-row">
        <GameInfo
          whitePlayer={queryGame?.game?.white_player?.username}
          blackPlayer={queryGame?.game?.black_player?.username}
          time="10 + 0"
          betValue={queryGame?.game?.bet_value}
          gameState={queryGame?.game?.state}
        />
        <Board
          boardOrientation={boardOrientation()}
          san={san}
          onMove={playMove}
        />
        <GameDisplay san={san} topPlayer={getUsernames().top} bottomPlayer={getUsernames().bottom} />
      </div>
    </div>
  );
};

export default Game;
