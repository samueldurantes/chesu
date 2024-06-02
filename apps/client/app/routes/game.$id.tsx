import type {
  ActionFunctionArgs,
  LoaderFunctionArgs,
  MetaFunction,
} from '@remix-run/node';
import { redirect } from '@remix-run/node';
import { Form, useLoaderData } from '@remix-run/react';

import api from '../adapters/api';
import Board from '../components/board';
import { Button } from '../components/ui/button';
import { getSession } from '../utils/createSession';

export const meta: MetaFunction = () => {
  return [
    { title: 'chesu' },
    {
      name: 'description',
      content: 'A platform to play chess',
    },
  ];
};

export const loader = async ({
  request,
}: LoaderFunctionArgs): Promise<{
  user?: {
    username: string;
  };
}> => {
  const session = await getSession(request.headers.get('Cookie'));

  if (!session.data.token) {
    return {};
  }

  const { data, error } = await api.GET('/user/me', {
    headers: {
      Authorization: session.data.token,
    },
  });

  if (error) {
    return {};
  }

  return {
    user: {
      username: data.user.username,
    },
  };
};

export const action = async ({ request }: ActionFunctionArgs) => {
  const session = await getSession(request.headers.get('Cookie'));

  if (!session.data.token) {
    return {};
  }

  const { data, error } = await api.POST('/game/create', {
    headers: {
      Authorization: session.data.token,
    },
    body: {
      game: {
        bet_value: 0,
      },
    },
  });

  if (error || !data) {
    return {
      error,
    };
  }

  return redirect(`/game/${data.game.id}`);
};

const Index = () => {
  const data = useLoaderData<typeof loader>();

  const getName = () => {
    if (!data.user) {
      return 'Opponent';
    }

    return data.user.username;
  };

  return (
    <div className="h-screen flex items-center justify-center gap-2">
      <div className="flex flex-col gap-4 w-full max-w-[750px] px-6">
        <div className="flex bg-slate-300 p-4 gap-3">
          <div className="bg-slate-800 h-[50px] w-[50px]"></div>
          <span>Opponent</span>
        </div>
        <Board boardOrientation="white" />
        <div className="flex bg-slate-300 p-4 gap-3">
          <div className="bg-slate-800 h-[50px] w-[50px]"></div>
          <span>{getName()}</span>
        </div>
        {data?.user && (
          <Form method="post">
            <Button className="w-full">Play</Button>
          </Form>
        )}
      </div>
    </div>
  );
};

export default Index;
