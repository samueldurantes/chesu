import { useNavigate } from 'react-router-dom';
import { useMutation } from '@tanstack/react-query';

import api from '../../api/api';
import { Button } from '../ui/Button';

const HomeActions = () => {
  const navigate = useNavigate();


  const { mutateAsync: mutatePairingGame, isPending: isPendingPairGame } = useMutation({
    mutationFn: async () => {
      const { data, error } = await api.POST('/game/pairing');

      if (error) {
        throw new Error(error.message);
      }

      return data;
    },
    onSuccess: (data) => navigate(`/game/${data.game_id}`),
    // TODO: Show a snackbar with the error message
    onError: (error) => console.log({ error }),
  });

  const { mutateAsync: mutateCreateGame, isPending } = useMutation({
    mutationFn: async () => {
      const { data, error } = await api.POST('/game/create');

      if (error) {
        throw new Error(error.message);
      }

      return data;
    },
    onSuccess: (data) => navigate(`/game/${data.game_id}`),
    // TODO: Show a snackbar with the error message
    onError: (error) => console.log({ error }),
  });

  return (
    <div className="flex flex-col max-w-[300px] w-full gap-4">
      <Button
        disabled={isPending}
        className="w-full bg-[#3aafff] text-white hover:bg-[#80cfff]"
        onClick={() => mutateCreateGame()}
      >
        Create a game
      </Button>
      <Button
        className="w-full bg-[#3aafff] text-white hover:bg-[#80cfff]"
        onClick={() => mutatePairingGame()}
        disabled={isPendingPairGame}
      >
        Play
      </Button>
      <Button
        className="w-full bg-[#3aafff] text-white hover:bg-[#80cfff]"
        disabled
      >
        Play with a computer
      </Button>
    </div>
  );
};

export default HomeActions;
