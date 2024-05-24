import { useState } from 'react';
import type { MetaFunction } from '@remix-run/node';
import { Chessboard } from 'react-chessboard';
import { Chess, Square } from 'chess.js';

export const meta: MetaFunction = () => {
  return [{ title: 'chesu' }];
};

interface Move { from: string, to: string, promotion?: string | undefined };

const Index = () => {
  const [game, setGame] = useState(new Chess());

  function playMove(move: string | Move) {
    const gameCopy = new Chess(game.fen());
    let result = gameCopy.move(move).san;

    if (result) {
      setGame(gameCopy);
      //Send fetch new_move 
    }
    return result;
  }


  function onDrop(sourceSquare: Square, targetSquare: Square): boolean {
    const move = playMove({
      from: sourceSquare,
      to: targetSquare,
    });

    return move != null;
  }

  return (
    <div className="container">
      <Chessboard showPromotionDialog={true} position={game.fen()} boardWidth={600} id=" BasicBoard" onPieceDrop={onDrop} />
    </div>
  );
};

export default Index;
