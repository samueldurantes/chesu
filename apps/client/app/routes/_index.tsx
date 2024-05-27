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
    <div className="container">
      <Link to="/game">Play a match</Link>
    </div>
  );
};

export default Index;
