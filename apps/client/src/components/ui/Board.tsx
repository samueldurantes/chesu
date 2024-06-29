import { Chess, Square } from 'chess.js';
import { Chessboard } from 'react-chessboard';

type Move = {
  from: string;
  to: string;
  promotion?: string;
};

type Props = {
  fen?: string;
  boardOrientation: 'white' | 'black';
  onMove?: (move: string) => void;
};

export const Board = (props: Props) => {
  const playMove = (move: string | Move) => {
    const gameCopy = new Chess(props.fen);
    const result = gameCopy.move(move).san;

    if (result) {
      props.onMove && props.onMove(result);
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
    <Chessboard
      boardOrientation={props.boardOrientation}
      showPromotionDialog={true}
      position={props.fen}
      onPieceDrop={onDrop}
    />
  );
};

