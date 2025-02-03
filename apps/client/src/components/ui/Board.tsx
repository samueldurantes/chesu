import { Chess, Square } from 'chess.js';
import { Chessboard } from 'react-chessboard';

type Move = {
  from: string;
  to: string;
  promotion?: string;
};

type Props = {
  san?: string[];
  boardOrientation: 'white' | 'black';
  onMove?: (move: string) => void;
};

const sanToFen = (sans: string[]): string => {
  const chess = new Chess();

  for (const san of sans) {
    chess.move(san);
  }

  return chess.fen();
};

export const Board = ({ san = [], ...props }: Props) => {
  const fen = sanToFen(san);

  const playMove = (move: string | Move) => {
    const gameCopy = new Chess(fen);
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
      clearPremovesOnRightClick={true}
      showPromotionDialog={true}
      position={fen}
      onPieceDrop={onDrop}
      areArrowsAllowed={true}
      arePremovesAllowed={true}
      boardWidth={800}
    />
  );
};
