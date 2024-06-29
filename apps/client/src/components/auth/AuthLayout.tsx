import { Navigate, Outlet } from 'react-router-dom';
import { useQuery } from "@tanstack/react-query";

import api from "../../api/api";

const AuthLayout = () => {
  const { data: query } = useQuery({
    queryKey: ["authLayout"],
    queryFn: async () => await api.GET('/user/me'),
    networkMode: "always",
    // @ts-ignore
    suspense: true,
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

