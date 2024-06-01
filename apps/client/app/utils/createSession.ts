import { createCookieSessionStorage } from '@remix-run/node';

type SessionData = {
  token: string;
};

type SessionFlashData = {
  error: string;
};

const { getSession, commitSession, destroySession } =
  createCookieSessionStorage<SessionData, SessionFlashData>({
    cookie: {
      name: 'CHESU_TOKEN',
      domain: 'localhost',
      httpOnly: true,
      maxAge: 6000000,
      path: '/',
      sameSite: 'lax',
      secrets: ['s3cret1'],
      secure: true,
    },
  });

export { getSession, commitSession, destroySession };
