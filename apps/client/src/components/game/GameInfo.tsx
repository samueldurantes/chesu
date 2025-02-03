
interface GameInfoProps {
  san: string[];
  bottomPlayer: string
  topPlayer: string
}

const piecesTable = [
  ["K", "♔", "♚"],
  ["Q", "♕", "♛"],
  ["R", "♖", "♜"],
  ["B", "♗", "♝"],
  ["N", "♘", "♞"],
]

const formatMove = (move: string, isWhite: boolean): string => {
  piecesTable.forEach(piece => { move = move?.replace(piece[0], piece[isWhite ? 2 : 1]) })

  return move
}


const MoveBox = ({ index, white, black }: { index: number, white: string, black?: string }) => {

  return (
    <div className="flex flex-row h-10 shadow-xl">
      <div className="h-full w-2/12 text-lg flex bg-[#242424] self-center items-center justify-center">{index + 1}</div>
      <div className={`pl-2 items-center w-5/12 text-lg flex bg-[#1e1e1e]`}>
        {formatMove(white, true)}
      </div>
      <div className={`pl-2 items-center w-5/12 text-lg flex bg-[#1e1e1e]`}>
        {formatMove(black || "", false)}
      </div>
    </div >
  )
}

const GameInfo = ({ san, bottomPlayer, topPlayer }: GameInfoProps) => {

  const moves: { key: number, white: string, black?: string }[] = san.map((_, i) => ({ key: i, white: san[i], black: san[i + 1] })).filter((_, i) => i % 2 == 0);

  return (
    <div className="flex flex-col self-center m-4">
      <div className="text-white w-72 rounded bg-[#303030]">
        <div className="p-4 font-sans border-b border-[#999]">{topPlayer}</div>
        <div className="overflow-auto h-96">
          {moves.map(({ key, white, black }, index) => <MoveBox key={key} index={index} white={white} black={black} />)}
        </div>
        <div className="p-4 font-sans border-t border-[#999]">{bottomPlayer}</div>
      </div>
    </div >
  );
};

export default GameInfo;
