import createClient from 'openapi-fetch';

import type { paths } from '../../schemas/api';

const api = createClient<paths>({
  baseUrl: 'http://localhost:3000',
  credentials: 'include',
});

export default api;

