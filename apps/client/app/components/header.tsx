import { useState } from 'react';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from './ui/dropdown-menu';
import { User, CreditCard, Eye, EyeOff, LogOut } from 'lucide-react';
import { Link } from '@remix-run/react';

import { Button } from './ui/button';

const Header = () => {
  const [balanceVisibility, setBalanceVisibity] = useState(true);

  return (
    <div className="container flex h-20 max-w-screen-2xl items-center justify-between">
      <div className="text-black font-kadwa font-bold text-5xl select-none">
        <Link to="/home">chesu</Link>
      </div>
      <div>
        <Button
          variant="ghost"
          className="mr-4"
          onClick={() => setBalanceVisibity(!balanceVisibility)}
        >
          <div className="text-lg font-kadwa">
            R$ {balanceVisibility ? '1023,00' : '****'}
          </div>
          {balanceVisibility ? (
            <Eye className="ml-2" />
          ) : (
            <EyeOff className="ml-2" />
          )}
        </Button>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="outline" className="">
              <div className="font-['Kadwa'] text-lg">tiagovsk</div>
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent className="w-56 mr-8">
            <DropdownMenuGroup>
              <DropdownMenuItem>
                <User className="mr-2 h-4 w-4" />
                <span>Profile</span>
              </DropdownMenuItem>
              <DropdownMenuItem>
                <CreditCard className="mr-2 h-4 w-4" />
                <span>Add credits</span>
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem>
                <LogOut className="mr-2 h-4 w-4" />
                <span>Log out</span>
              </DropdownMenuItem>
            </DropdownMenuGroup>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </div>
  );
};

export default Header;
