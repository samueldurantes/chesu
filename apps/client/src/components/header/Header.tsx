import { DropdownMenuGroup, DropdownMenuSeparator } from "@radix-ui/react-dropdown-menu";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuLabel, DropdownMenuTrigger } from "../ui/dropdown-menu";
import { useNavigate } from "react-router-dom";
import { useState } from "react";
import DepositDialog from "../wallet/DepositDialog";
import { useMutation } from "@tanstack/react-query";
import api from "../../api/api";
import WithdrawDialog from "../wallet/WithdrawDeposit";

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
  const [openDeposit, setOpenDeposit] = useState(false);
  const [openWithdraw, setOpenWithdraw] = useState(false);

  const navigate = useNavigate();

  const { mutateAsync: logout } = useMutation({
    mutationFn: async () => { await api.GET('/auth/logout'); },
    onSuccess: () => navigate('/login'),
  });

  return (
    <div className="h-32 w-screen flex items-center justify-between gap-2">
      <div
        className="select-none hover:cursor-pointer text-white px-6 m-8 text-4xl font-teko font-medium"
        onClick={() => { navigate('/') }}
      >/ Chesu</div>
      <DropdownMenu>
        <DropdownMenuTrigger className="hover:cursor-pointer select-none m-8 px-6 text-white text-xl focus:outline-none font-sans">
          {user?.username}
        </DropdownMenuTrigger>
        <DropdownMenuContent className="bg-white m-8 mt-2 rounded-lg p-4">
          <DropdownMenuLabel className="text-md flex gap-16 mb-2 pt-0">
            Balance <p className="font-light">{user?.balance} sats</p>
          </DropdownMenuLabel>
          <DropdownMenuSeparator />
          <DropdownMenuGroup>
            <DropdownMenuItem
              className="hover:cursor-pointer font-light p-1 pl-2 text-md"
              onClick={() => { setOpenDeposit(true) }}
            >
              Deposit
            </DropdownMenuItem>

            <DropdownMenuItem
              className="hover:cursor-pointer font-light p-1 pl-2 text-md"
              onClick={() => { setOpenWithdraw(true) }}
            >
              Withdraw
            </DropdownMenuItem>
          </DropdownMenuGroup>
          <DropdownMenuItem
            className="hover:cursor-pointer font-light p-1 pl-2 text-md"
            onClick={() => logout()}
          >
            Exit
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
      <DepositDialog open={openDeposit} setOpen={setOpenDeposit} />
      <WithdrawDialog open={openWithdraw} setOpen={setOpenWithdraw} />
    </div >
  );
};

export default Header;
