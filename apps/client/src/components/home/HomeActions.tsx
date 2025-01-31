import { useNavigate } from 'react-router-dom';
import { useMutation } from '@tanstack/react-query';
import { v4 as uuidv4 } from 'uuid'

import api from '../../api/api';
import { Button } from '../ui/Button';

const HomeActions = () => {
  const navigate = useNavigate();


  const { mutateAsync: mutatePairingGame, isPending } = useMutation({
    mutationFn: async (key: string) => {
      console.log(key)
      const { data, error } = await api.POST('/game/pairing', {
        body: {
          key
        },
      });

      if (error) throw new Error(error.message);

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
        onClick={() => mutatePairingGame(`w-10-0-0-${uuidv4()}`)}
      >
        Create a game
      </Button>
      <Button
        className="w-full bg-[#3aafff] text-white hover:bg-[#80cfff]"
        onClick={() => mutatePairingGame("w-10-0-0")}
        disabled={isPending}
      >
        Play
      </Button>
      <Button
        className="w-full bg-[#3aafff] text-white hover:bg-[#80cfff]"
        disabled
      >
        Play with a computer
      </Button>
    </div >
  );
};

export default HomeActions;
