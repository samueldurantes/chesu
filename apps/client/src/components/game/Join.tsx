import { useNavigate } from 'react-router-dom';
import { useEffect } from 'react';
import { useMutation } from '@tanstack/react-query';
import { useParams } from 'react-router-dom';

import api from '../../api/api';

const Join = () => {
  const navigate = useNavigate();
  const params = useParams();


  const { mutateAsync: mutateJoinGame } = useMutation({
    mutationFn: async () => {
      const { data, error } = await api.POST('/game/{id}', {
        params: {
          path: {
            id: params.id as string,
          },
        },
      });

      if (error) {
        throw new Error(error.message);
      }

      return data;
    },
    onSuccess: (data) => navigate(`/game/${data.game}`),
    onError: (error) => console.log({ error }),
  });

  useEffect(() => {
    mutateJoinGame()
  }, [])

  return (
    <div className="flex items-center justify-center h-screen bg-gray-100">
      <div className="text-center">
        <p className="text-xl mb-4">Waiting for the game to start...</p>
      </div>
    </div>
  );
};

export default Join;
