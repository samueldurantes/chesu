import { useEffect } from 'react';
import { z, ZodError } from 'zod';
import { useFormik, FormikProvider } from 'formik';
import { useQuery, useMutation } from '@tanstack/react-query';
import { useNavigate } from 'react-router-dom';

import api from '../../api/api';
import { Input } from '../ui/Input';
import { Label } from '../ui/Label';
import { Card, CardHeader, CardTitle, CardContent } from '../ui/Card';
import { Button } from '../ui/Button';

const schema = z.object({
  email: z.string().email(),
  password: z.string(),
});

type Values = z.infer<typeof schema>;

const Login = () => {
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
    queryKey: ['auth/login'],
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

  const { mutateAsync } = useMutation({
    mutationFn: async (data: Values) => {
      const response = await api.POST('/auth/login', {
        body: {
          user: {
            email: data.email,
            password: data.password,
          },
        },
      });

      return response.data;
    },
    onSuccess: () => navigate('/'),
  });

  const formikbag = useFormik({
    initialValues: {
      email: '',
      password: '',
    },
    validate,
    onSubmit: (values: Values) => {
      mutateAsync(values);
    },
  });

  const { handleSubmit } = formikbag;

  return (
    <div className="h-screen flex items-center justify-center bg-[#121212] px-6">
      <Card className="bg-[#1e1e1e] max-w-screen-sm w-full">
        <CardHeader className="text-white gap-5">
          <CardTitle>Login</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <FormikProvider value={formikbag} >
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
                Login
              </Button>
            </FormikProvider>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

export default Login;
