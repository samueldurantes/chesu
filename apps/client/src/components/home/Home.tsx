import { useQuery } from '@tanstack/react-query';

import api from '../../api/api';
import { Board } from '../ui/Board';
import { Card } from '../ui/Card';
import HomeActions from './HomeActions';
import Header from '../header/Header';

const Home = () => {
  const { data: query } = useQuery({
    queryKey: ['user/me'],
    queryFn: () => api.GET('/user/me'),
  });

  return (
    <div className="flex min-h-screen flex-col items-center gap-2 bg-[#121212]">
      <Header user={query?.data?.user} />

      <div className="w-screen mb-8 flex items-center justify-center gap-2">
        <div className="flex flex-col gap-4 w-full max-w-[750px] px-6">
          <Card className="flex items-center gap-2 p-4 bg-[#1e1e1e]">
            <div className="bg-white h-[50px] w-[50px]"></div>
            <span className="text-white">Opponent</span>
          </Card>
          <Board boardOrientation="white" />
          <Card className="flex items-center gap-2 p-4 bg-[#1e1e1e]">
            <div className="bg-white h-[50px] w-[50px]"></div>
            <span className="text-white">{query?.data?.user?.username}</span>
          </Card>
        </div>
        <HomeActions />
      </div>
    </div >
  );
};

export default Home;
