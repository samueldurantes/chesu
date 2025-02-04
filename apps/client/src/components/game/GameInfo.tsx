interface GameInfoProps {
  time: string
  betValue: number
  whitePlayer?: string
  blackPlayer?: string
  gameState?: string
}

const GameInfo = ({ gameState, time, betValue, whitePlayer, blackPlayer }: GameInfoProps) => {
  const parsedGameState = () => {
    switch (gameState) {
      case "WhiteWin": return "White is victorious"
      case "BlackWin": return "Black is victorious"
      case "Draw": return "Draw";
    }
  }

  return (
    <div className="flex flex-col self-center m-4">
      <div className="p-4 text-white w-72 rounded bg-[#303030] flex flex-col">
        <div className="font-sans text-lg font-semibold self-center m-1">Game</div>
        <div className="font-sans self-center m-2">{time} 🞄 {betValue} sats</div>
        <div className="font-sans m-1 mx-5 flex gap-2 items-center">
          <div className="size-4 bg-white rounded-full" />
          {whitePlayer}
        </div>
        <div className="font-sans m-1 mb-2 mx-5 flex gap-2 items-center">
          <div className="size-4 border rounded-full" />
          {blackPlayer}
        </div>
        {parsedGameState() &&
          <>
            <div className="m-4 mb-2 pt-4 self-center border-t px-10">
              {parsedGameState()}
            </div>
          </>
        }
      </div>
    </div >
  );
};

export default GameInfo;
