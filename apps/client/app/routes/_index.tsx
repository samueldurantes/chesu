import type { MetaFunction } from '@remix-run/node';
import { Link } from '@remix-run/react';

export const meta: MetaFunction = () => {
  return [{ title: 'chesu' }, {
    name: "description",
    content: "A platform to play chess",
  }];
};

const Index = () => {
  return (
    <div>
      <Link to="/home" className="">Go to home</Link>
    </div >
  );
};

export default Index;
