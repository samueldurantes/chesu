import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "../ui/dialog";
import QRCode from "react-qr-code";

import { useState, useEffect } from 'react';
import { z, ZodError } from 'zod';
import { useFormik, FormikProvider } from 'formik';
import { useMutation } from '@tanstack/react-query';

import api from '../../api/api';
import { Input } from '../ui/Input';
import { Label } from '../ui/Label';
import { Button } from '../ui/Button';
import { Copy } from "lucide-react";

const schema = z.object({
  amount: z.number({ message: "Amount is missing" }).int("Amount need to be an integer").gt(0, "Amount need to be greater than 0")
});

type Values = z.infer<typeof schema>;

interface DepositProps {
  open: boolean;
  setOpen: (isOpen: boolean) => void;
}

const DepositDialog = ({ open, setOpen }: DepositProps) => {
  const [invoice, setInvoice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const validate = (values: Values) => {
    try {
      schema.parse(values);
    } catch (error) {
      if (error instanceof ZodError) return error.formErrors.fieldErrors;
    }
  };

  const { mutateAsync: checkInvoice } = useMutation({
    mutationFn: async () => {
      console.log("Req")
      const { data, error } = await api.GET('/invoice/check');

      if (error) throw new Error(error.message);

      return data;
    },
    onSuccess: (data) => {
      if (data.invoice == invoice) {
        console.log("Payed")
        localStorage.removeItem("invoice");
        setInvoice(null);
        setError(null);
      }
    },
  });

  useEffect(() => {
    setInvoice(localStorage.getItem("invoice"))
    setInterval(() => { if (localStorage.getItem("invoice")) checkInvoice(); }, 3000)
  }, [])

  const { mutateAsync: mutate } = useMutation({
    mutationFn: async ({ amount }: Values) => {
      const { data, error } = await api.POST('/invoice/create', {
        body: {
          amount,
        },
      });

      if (error) throw new Error(error.message);

      return data;
    },
    onSuccess: data => { setInvoice(data.invoice); localStorage.setItem("invoice", data.invoice); },
    onError: (error) => setError(error.message),
  });

  const formikbag = useFormik({
    initialValues: {
      amount: 1,
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
          <DialogTitle>Deposit</DialogTitle>
          <DialogDescription>Add satoshis to your account. </DialogDescription>
        </DialogHeader>
        {invoice == null ?
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
                    Amount
                  </Label>
                  <Input
                    className="border text-right"
                    name="amount"
                    type="number"
                    placeholder="Amount"
                    disabled={!open}
                  />
                </div>
                <Button
                  className="w-full bg-[#3aafff] text-white hover:bg-[#80cfff]"
                  type="submit"
                  onClick={() => handleSubmit()}
                >
                  Create invoice
                </Button>
              </FormikProvider>
            </div>
          </div>
          :
          <div className="flex flex-col justify-center items-center">
            <QRCode className="m-4 w-3/5" value={invoice} />
            <div
              className="bg-[#3aafff] m-4 w-3/5 p-2 rounded-md flex justify-center text-white items-center hover:bg-[#80cfff]"
              onClick={async () => { await navigator.clipboard.writeText(invoice); }}>
              <Copy className="" />
            </div>
          </div>
        }
      </DialogContent>
    </Dialog >
  );
};

export default DepositDialog;
