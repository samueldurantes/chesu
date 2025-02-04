interface GameInfoProps {
  time: string
  betValue: number
  whitePlayer?: string
  blackPlayer?: string
}

const GameInfo = ({ time, betValue, whitePlayer, blackPlayer }: GameInfoProps) => {

  return (
    <div className="flex flex-col self-center m-4">
      <div className="p-4 text-white w-72 rounded bg-[#303030]">
        <div className="font-sans">Time: {time} </div>
        <div className="font-sans">Bet value: {betValue} sats</div>
        {whitePlayer != "Waiting player..." && blackPlayer == "Waiting player..." &&
          <>
            <div className="font-sans">{whitePlayer}</div>
            <div className="font-sans">{blackPlayer}</div>
          </>
        }
      </div>
    </div >
  );
};

export default GameInfo;
