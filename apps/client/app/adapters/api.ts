import createClient from 'openapi-fetch';

import type { paths } from '../../schemas/api';
import { getSession } from '../utils/createSession';

const unauthorized = createClient<paths>({
  baseUrl: 'http://localhost:3000',
});

const authorized = async (request: Request) => {
  const session = await getSession(request.headers.get('Cookie'));

  return createClient<paths>({
    baseUrl: 'http://localhost:3000',
    headers: {
      Authorization: session.get('token'),
    },
  });
};

export default {
  unauthorized,
  authorized,
};
