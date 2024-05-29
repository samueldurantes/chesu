import { ReactNode } from 'react';
import { Button } from './ui/button';

type Parameters = {
  target: string;
  value: string;
  fn: Function;
  children: ReactNode;
};

const Radio = ({ target, value, fn, children }: Parameters) => {
  return (
    <Button
      onClick={() => fn(target)}
      variant={target != value ? 'outline' : undefined}
      className={`w-40 ${
        target == value
          ? 'text-white bg-black hover:bg-black'
          : 'hover:bg-black hover:text-white'
      }`}
    >
      {children}
    </Button>
  );
};

export default Radio;
