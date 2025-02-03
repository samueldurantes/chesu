import { useQuery } from '@tanstack/react-query';
import { useMutation } from '@tanstack/react-query';
import { useNavigate } from 'react-router-dom';
import { v4 as uuidv4 } from 'uuid'

import api from '../../api/api';
import Header from '../header/Header';

interface ActionProps {
  gameRequest: () => void;
  title: string;
  subtitle: string;
}

const Home = () => {
  const navigate = useNavigate();

  const { data: query } = useQuery({
    queryKey: ['user/me'],
    queryFn: () => api.GET('/user/me'),
  });

  const { mutateAsync: mutatePairingGame, isPending: _ } = useMutation({
    mutationFn: async (key: string) => {
      console.log(key)

      const { data, error } = await api.POST('/game/pairing', {
        body: { key },
      });

      if (error) throw new Error(error.message);

      return data;
    },
    onSuccess: (data) => navigate(`/game/${data.game_id}`),
    // TODO: Show a snackbar with the error message
    onError: (error) => console.log({ error }),
  });

  const actions: ActionProps[] = [
    { gameRequest: () => { mutatePairingGame(`w-1-0-30`) }, title: "1 + 0", subtitle: "30 sats" },
    { gameRequest: () => { mutatePairingGame(`w-1-2-50`) }, title: "1 + 2", subtitle: "50 sats" },
    { gameRequest: () => { mutatePairingGame(`w-1-2-100`) }, title: "1 + 2", subtitle: "100 sats" },

    { gameRequest: () => { mutatePairingGame(`w-3-2-30`) }, title: "3 + 2", subtitle: "30 sats" },
    { gameRequest: () => { mutatePairingGame(`w-3-2-50`) }, title: "3 + 2", subtitle: "50 sats" },
    { gameRequest: () => { mutatePairingGame(`w-5-0-50`) }, title: "5 + 0", subtitle: "50 sats" },

    { gameRequest: () => { mutatePairingGame(`w-10-0-100`) }, title: "10 + 0", subtitle: "100 sats" },
    { gameRequest: () => { mutatePairingGame(`w-10-0-0-${uuidv4()}`) }, title: "Play", subtitle: "with friend" },
    { gameRequest: () => { }, title: "Create", subtitle: "Custom Game" },
  ]

  return (
    <div className="flex min-h-screen flex-col items-center gap-2 bg-[#121212]">
      <Header user={query?.data?.user} />

      <div className="w-screen mb-8 flex items-center justify-center gap-2">
        <div className="grid grid-cols-3 gap-12">
          {actions.map(({ gameRequest, title, subtitle }, key) => (
            <div
              key={key}
              onClick={gameRequest}
              className="gap-4 bg-white rounded-lg size-44 shadow-lg flex flex-col justify-center items-center transform transition-transform duration-300 ease-in-out hover:-translate-y-2"
            >
              <div className="font-normal text-2xl">{title}</div>
              <div className="font-normal text-lg">{subtitle}</div>
            </div>
          ))}

        </div>
      </div>
    </div >
  );
};

export default Home;
