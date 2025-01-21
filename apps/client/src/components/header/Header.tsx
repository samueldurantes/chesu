import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuLabel, DropdownMenuTrigger } from "@radix-ui/react-dropdown-menu";
import { useNavigate } from "react-router-dom";

interface UserInfo {
  id: string;
  username: string;
  email: string
  balance: number
}

interface HeaderProps {
  user?: UserInfo
}

const Header = ({ user }: HeaderProps) => {
  const navigate = useNavigate();

  return (
    <div className="h-32 w-screen flex items-center justify-between gap-2">
      <div
        className="select-none hover:cursor-pointer text-white pl-6 pr-6 m-8 text-4xl font-teko font-medium"
        onClick={() => { navigate('/') }}
      >/ Chesu</div>
      <DropdownMenu>
        <DropdownMenuTrigger className="hover:cursor-pointer m-8 p-6 pt-2 pb-2 text-white text-xl focus:outline-none font-sans">{user?.username}</DropdownMenuTrigger>
        <DropdownMenuContent className="m-8 mt-2 rounded-lg bg-white p-4 text-md">
          <DropdownMenuLabel className=""></DropdownMenuLabel>
          <DropdownMenuLabel className="flex gap-16 mb-2 pl-3 p-2 pt-1">
            <p className="font-medium">Balance </p>
            <p className="font-extralight">{user?.balance} sats</p>
          </DropdownMenuLabel>
          <DropdownMenuItem
            className="hover:cursor-pointer select-none pl-3 p-2 mb-1 mt-1 w-full hover:outline-none rounded-md hover:bg-gray-200 font-light"
            onClick={() => { }}
          >
            Deposit
          </DropdownMenuItem>

          <DropdownMenuItem
            className="hover:cursor-pointer select-none pl-3 p-2 mb-1 mt-1 w-full hover:outline-none rounded-md hover:bg-gray-100 font-light"
            onClick={() => { }}
          >
            Withdraw
          </DropdownMenuItem>

          <DropdownMenuItem
            className="hover:cursor-pointer select-none pl-3 p-2 mb-1 mt-1 w-full hover:outline-none rounded-md hover:bg-red-500 hover:text-white font-light"
            onClick={() => { }}
          >
            Exit
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
};

export default Header;
