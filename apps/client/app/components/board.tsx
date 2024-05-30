import { useState } from 'react';
import { Chessboard } from 'react-chessboard';
import { Chess, Square } from 'chess.js';

type Move = {
  from: string;
  to: string;
  promotion?: string;
};

type Props = {
  boardOrientation: 'white' | 'black';
};

const Board = (props: Props) => {
  const [game, setGame] = useState(new Chess());

  const playMove = (move: string | Move) => {
    const gameCopy = new Chess(game.fen());
    let result;
    try {
      result = gameCopy.move(move).san;
    } catch (e) {}

    if (result) {
      setGame(gameCopy);

      //Send fetch new_move
    }
    return result;
  };

  const onDrop = (sourceSquare: Square, targetSquare: Square): boolean => {
    const move = playMove({
      from: sourceSquare,
      to: targetSquare,
    });

    return move != null;
  };

  return (
    <div className="flex justify-center items-center">
      <Chessboard
        boardWidth={1000}
        boardOrientation={props.boardOrientation}
        showPromotionDialog={true}
        position={game.fen()}
        onPieceDrop={onDrop}
      />
    </div>
  );
};

export default Board;
