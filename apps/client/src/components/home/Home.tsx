import { Board } from '../ui/Board';
import { Button } from '../ui/Button';
import { Card } from '../ui/Card';

const Home = () => {
	return (
  	<div className="h-screen flex items-center justify-center gap-2 bg-[#121212]">
			<Card className="flex gap-2 p-4 max-w-[300px] w-full h-[90%] bg-[#1e1e1e]">
				<span className="text-md text-white">Recent games:</span>
			</Card>

  	  <div className="flex flex-col gap-4 w-full max-w-[750px] px-6">
				<Card className="flex items-center gap-2 p-4 bg-[#1e1e1e]">
					<div className="bg-white h-[50px] w-[50px]"></div>
					<span className="text-white">Opponent</span>
				</Card>

  	    <Board boardOrientation="white" />

				<Card className="flex items-center gap-2 p-4 bg-[#1e1e1e]">
					<div className="bg-white h-[50px] w-[50px]"></div>
					<span className="text-white">Opponent</span>
				</Card>
  	  </div>

			<div className="flex flex-col max-w-[300px] w-full gap-4">
				<Button className="w-full bg-[#3aafff] text-white hover:bg-[#80cfff]">Create a game</Button>
				<Button className="w-full bg-[#3aafff] text-white hover:bg-[#80cfff]">Play with a friend</Button>
				<Button className="w-full bg-[#3aafff] text-white hover:bg-[#80cfff]">Play with a computer</Button>
			</div>
  	</div>
	);
};

export default Home;

