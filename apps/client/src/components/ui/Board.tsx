import { Chess, Square } from 'chess.js';
import { Chessboard } from 'react-chessboard';

type Move = {
  from: string;
  to: string;
  promotion?: string;
};

type Props = {
  san: string[];
  boardOrientation: 'white' | 'black';
  onMove: (move: string) => void;
  isPlayable: boolean
};

const sanToFen = (sans: string[]): string => {
  const chess = new Chess();

  for (const san of sans) chess.move(san);

  return chess.fen();
};

const playMove = (fen: string, move: Move) => { return (new Chess(fen)).move(move).san; };

export const Board = ({ san = [], ...props }: Props) => {
  const fen = sanToFen(san);

  const onDrop = (sourceSquare: Square, targetSquare: Square): boolean => {
    const move = playMove(
      fen,
      {
        from: sourceSquare,
        to: targetSquare,
      });

    if (!props.isPlayable) return false;


    props.onMove(move);

    return true
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
