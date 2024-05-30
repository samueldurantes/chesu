import type { MetaFunction } from '@remix-run/node';

import Board from '../components/board';
import NewMatch from '../components/new-match';

export const meta: MetaFunction = () => {
  return [
    { title: 'chesu' },
    {
      name: 'description',
      content: 'A platform to play chess',
    },
  ];
};

const Index = () => {
  return (
    <div className="h-screen flex">
      <div className="h-screen w-[70%] flex items-center justify-center border">
        <Board boardOrientation="white" />
      </div>
      <div className="h-screen flex flex-col w-[30%] border">
        <NewMatch />
      </div>
    </div>
  );
};

export default Index;
