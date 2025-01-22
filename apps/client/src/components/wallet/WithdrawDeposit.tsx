import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "../ui/dialog";
import { useToast } from "../../hooks/use-toast"

import { useState } from 'react';
import { z, ZodError } from 'zod';
import { useFormik, FormikProvider } from 'formik';
import { useMutation, useQueryClient } from '@tanstack/react-query';

import api from '../../api/api';
import { Input } from '../ui/Input';
import { Label } from '../ui/Label';
import { Button } from '../ui/Button';

const schema = z.object({
  invoice: z.string()
});

type Values = z.infer<typeof schema>;

interface WithdrawProps {
  open: boolean;
  setOpen: (isOpen: boolean) => void;
}

const WithdrawDialog = ({ open, setOpen }: WithdrawProps) => {
  const [error, setError] = useState<string | null>(null);
  const queryClient = useQueryClient();
  const { toast } = useToast();


  const validate = (values: Values) => {
    try {
      schema.parse(values);
    } catch (error) {
      if (error instanceof ZodError) return error.formErrors.fieldErrors;
    }
  };

  const { mutateAsync: mutate } = useMutation({
    mutationFn: async ({ invoice }: Values) => {
      const { data: _, error } = await api.POST('/invoice/withdraw', {
        body: { invoice, },
      });
      if (error) throw new Error(error.message);
    },
    onSuccess: () => {
      queryClient.refetchQueries({ queryKey: ['user/me'] });
      toast({
        title: "Withdraw was completed",
        className: "bg-green-500 text-white border-0",
        type: "background",
      });
      setError(null);
      setOpen(false);
    },
    onError: (error) => setError(error.message),
  });

  const formikbag = useFormik({
    initialValues: {
      invoice: '',
    },
    validate,
    onSubmit: (values: Values, { resetForm }) => {
      mutate(values);
      resetForm()
    },
  });

  const { handleSubmit } = formikbag;

  return (
    <Dialog open={open} onOpenChange={setOpen} >
      <DialogContent className="bg-white">
        <DialogHeader>
          <DialogTitle>Withdraw</DialogTitle>
          <DialogDescription>Withdraw you satoshis.</DialogDescription>
        </DialogHeader>
        <div>
          {error &&
            <div className="w-full bg-red-500 p-3 rounded text-white" >
              <p>{error}</p>
            </div>
          }
          <div className="space-y-4">
            <FormikProvider value={formikbag}>
              <div className="space-y-2">
                <Label className="" htmlFor="amount">
                  Invoice
                </Label>
                <Input
                  className="border text-right"
                  name="invoice"
                  placeholder="Invoice"
                  disabled={!open}
                />
              </div>
              <Button
                className="w-full bg-[#3aafff] text-white hover:bg-[#80cfff]"
                type="submit"
                onClick={() => handleSubmit()}
              >
                Send Invoice
              </Button>
            </FormikProvider>
          </div>
        </div>
      </DialogContent>
    </Dialog >
  );
};

export default WithdrawDialog;
