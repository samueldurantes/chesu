import type { MetaFunction } from '@remix-run/node';

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
    <div className="flex">
      <div className="h-screen bg-black w-[70%]">h1</div>
      <div className="h-screen flex flex-col w-[30%]">
        <div className="h-[50%] bg-slate-900">h2</div>
        <div className="h-[50%] bg-gray-800">h3</div>
      </div>
    </div>
  );
};

export default Index;
