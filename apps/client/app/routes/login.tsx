/* eslint-disable @typescript-eslint/ban-ts-comment */
import { z } from 'zod';
import {
  ActionFunctionArgs,
  LoaderFunctionArgs,
  MetaFunction,
  redirect,
} from '@remix-run/node';
import { Form, useActionData } from '@remix-run/react';

import { Label } from '../components/ui/label';
import { Input } from '../components/ui/input';
import { Button } from '../components/ui/button';
import {
  CardTitle,
  CardHeader,
  CardContent,
  Card,
} from '../components/ui/card';
import { validationAction } from '../utils/validationAction';
import { commitSession, getSession } from '../utils/createSession';
import api from '../adapters/api';

export const meta: MetaFunction = () => {
  return [{ title: 'Login | Chesu' }];
};

const schema = z.object({
  email: z.string().email(),
  password: z.string(),
});

export const loader = async ({ request }: LoaderFunctionArgs) => {
  const session = await getSession(request.headers.get('Cookie'));

  if (!session.data.token) {
    return null;
  }

  return redirect('/', {
    headers: {
      'Set-Cookie': await commitSession(session),
    },
  });
};

export const action = async ({ request }: ActionFunctionArgs) => {
  const { formData, errors, success } = await validationAction({
    request,
    schema,
  });

  const session = await getSession(request.headers.get('Cookie'));

  if (!success) {
    return {
      errors,
    };
  }

  const { email, password } = formData;

  const { data, error } = await api.POST('/auth/login', {
    body: {
      user: {
        email: email.toString(),
        password: password.toString(),
      },
    },
  });

  if (error || !data) {
    return {
      // @ts-ignore
      error: error?.error,
    };
  }

  session.set('token', data.user.token);

  return redirect('/', {
    headers: {
      'Set-Cookie': await commitSession(session),
    },
  });
};

const Login = () => {
  const data = useActionData<typeof action>();

  return (
    <div className="h-screen flex items-center justify-center bg-[#121212] px-6">
      <Card className="bg-[#1e1e1e] max-w-screen-sm w-full">
        <CardHeader className="text-white gap-5">
          {/* @ts-ignore */}
          {data?.error ? (
            <div className="w-full bg-red-500 p-3 rounded">
              {/* @ts-ignore */}
              <p>{data?.error}</p>
            </div>
          ) : null}
          <CardTitle>Login</CardTitle>
        </CardHeader>
        <CardContent>
          <Form method="post" className="space-y-4">
            <div className="space-y-2">
              <Label className="text-white" htmlFor="email">
                Email
              </Label>
              <Input
                className="bg-[#333] text-white placeholder-white"
                name="email"
                placeholder="Email"
                error={data?.errors?.email}
              />
            </div>
            <div className="space-y-2">
              <Label className="text-white" htmlFor="password">
                Password
              </Label>
              <Input
                className="bg-[#333] text-white placeholder-white border"
                name="password"
                placeholder="Password"
                type="password"
                error={data?.errors?.password}
              />
            </div>
            <Button
              className="w-full bg-[#3aafff] text-white hover:bg-[#80cfff]"
              type="submit"
            >
              Login
            </Button>
          </Form>
        </CardContent>
      </Card>
    </div>
  );
};

export default Login;
