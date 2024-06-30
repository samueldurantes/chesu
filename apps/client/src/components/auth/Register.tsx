import { useState, useEffect } from 'react';
import { z, ZodError } from 'zod';
import { useFormik, FormikProvider } from 'formik';
import { useQuery, useMutation } from '@tanstack/react-query';

import api from '../../api/api';
import { Input } from '../ui/Input';
import { Label } from '../ui/Label';
import { Card, CardHeader, CardTitle, CardContent } from '../ui/Card';
import { Button } from '../ui/Button';
import { useNavigate } from 'react-router-dom';

const schema = z.object({
  username: z
    .string()
    .min(2, { message: 'Username must contain at least 2 characters' }),
  email: z.string().email(),
  password: z
    .string()
    .min(8, { message: 'Password must contain at least 8 characters' }),
});

type Values = z.infer<typeof schema>;

const Register = () => {
  const [error, setError] = useState<string | null>(null);
  const navigate = useNavigate();

  const validate = (values: Values) => {
    try {
      schema.parse(values);
    } catch (error) {
      if (error instanceof ZodError) {
        return error.formErrors.fieldErrors;
      }
    }
  };

  const { data: query } = useQuery({
    queryKey: ['user/me'],
    queryFn: () => api.GET('/user/me'),
  });

  const redirectIfNotAuthenticated = () => {
    if (query?.data?.user) {
      navigate('/');
    }
  };

  useEffect(() => {
    redirectIfNotAuthenticated();
  });

  const { mutateAsync: mutate } = useMutation({
    mutationFn: async (user: Values) => {
      const { data, error } = await api.POST('/auth/register', {
        body: {
          user,
        },
      });

      if (error) {
        throw new Error(error.message);
      }

      return {
        data,
      };
    },
    onSuccess: () => navigate('/'),
    onError: (error) => setError(error.message),
  });

  const formikbag = useFormik({
    initialValues: {
      username: '',
      email: '',
      password: '',
    },
    validate,
    onSubmit: (values: Values) => {
      mutate(values);
    },
  });

  const { handleSubmit } = formikbag;

  return (
    <div className="h-screen flex items-center justify-center bg-[#121212] px-6">
      <Card className="bg-[#1e1e1e] max-w-screen-sm w-full">
        <CardHeader className="text-white gap-5">
          {error ? (
            <div className="w-full bg-red-500 p-3 rounded">
              <p>{error}</p>
            </div>
          ) : null}
          <CardTitle>Register</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <FormikProvider value={formikbag}>
              <div className="space-y-2">
                <Label className="text-white" htmlFor="username">
                  Username
                </Label>
                <Input
                  className="bg-[#333] text-white placeholder-white"
                  name="username"
                  placeholder="Username"
                />
              </div>
              <div className="space-y-2">
                <Label className="text-white" htmlFor="email">
                  Email
                </Label>
                <Input
                  className="bg-[#333] text-white placeholder-white"
                  name="email"
                  placeholder="Email"
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
                />
              </div>
              <Button
                className="w-full bg-[#3aafff] text-white hover:bg-[#80cfff]"
                type="submit"
                onClick={() => handleSubmit()}
              >
                Register
              </Button>
            </FormikProvider>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

export default Register;
