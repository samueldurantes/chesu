import { useState } from 'react';
import { Chessboard } from 'react-chessboard';
import { Chess, Square } from 'chess.js';

interface Move { from: string, to: string, promotion?: string | undefined };

const Board = () => {
  const [game, setGame] = useState(new Chess());

  function playMove(move: string | Move) {
    const gameCopy = new Chess(game.fen());
    let result;
    try {
      result = gameCopy.move(move).san;
    } catch (e) { }

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

export default Board;
