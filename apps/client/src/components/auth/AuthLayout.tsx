import { Navigate, Outlet } from 'react-router-dom';
import { useSuspenseQuery } from '@tanstack/react-query';

import api from '../../api/api';

const AuthLayout = () => {
  const { data: query } = useSuspenseQuery({
    queryKey: ['auth'],
    queryFn: async () => await api.GET('/user/me'),
    networkMode: 'always',
  });

  const user = query?.data?.user;

  if (!user) {
    return <Navigate to="/login" />;
  }

  return (
    <>
      <Outlet />
    </>
  );
};

export default AuthLayout;
