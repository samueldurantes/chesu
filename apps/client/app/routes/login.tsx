import { z } from 'zod';
import { ActionFunction, MetaFunction } from '@remix-run/node';
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

export const meta: MetaFunction = () => {
  return [{ title: 'Login | Chesu' }];
};

const schema = z.object({
  email: z.string().email(),
  password: z.string(),
});

export const action: ActionFunction = async ({ request }) => {
  const { formData, errors } = await validationAction({ request, schema });

  if (errors) {
    return {
      errors,
    };
  }

  const { email, password } = formData;

  return {
    email,
    password,
  };
};

const Login = () => {
  const data = useActionData<typeof action>();

  return (
    <div className="h-screen flex items-center justify-center bg-[#121212] px-6">
      <Card className="bg-[#1e1e1e] max-w-screen-sm w-full">
        <CardHeader className="text-white">
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
