import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "../ui/dialog";
import QRCode from "react-qr-code";
import { useToast } from "../../hooks/use-toast"

import { useState, useEffect } from 'react';
import { z, ZodError } from 'zod';
import { useFormik, FormikProvider } from 'formik';
import { useMutation, useQueryClient } from '@tanstack/react-query';

import api from '../../api/api';
import { Input } from '../ui/Input';
import { Label } from '../ui/Label';
import { Button } from '../ui/Button';
import { Copy, X } from "lucide-react";

const INVOICE_EXP_TIME = 60_000;
const POOLING_INTERVAL_TIME = 3_000;

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
  const queryClient = useQueryClient();
  const { toast } = useToast();
  // TODO: generate invoice in client side

  const validate = (values: Values) => {
    try {
      schema.parse(values);
    } catch (error) {
      if (error instanceof ZodError) return error.formErrors.fieldErrors;
    }
  };

  const clearDialog = () => {
    localStorage.removeItem("invoice");
    localStorage.removeItem("invoice-exp-time");
    setInvoice(null);
    setError(null);
  }

  const { mutateAsync: checkInvoice } = useMutation({
    mutationFn: async () => {
      const { data, error } = await api.GET('/invoice/check');

      if (error) throw new Error(error.message);

      return data;
    },
    onSuccess: (data) => {
      if (data.invoice == invoice) {
        queryClient.refetchQueries({ queryKey: ['user/me'] });
        toast({
          title: "Payment was completed",
          className: "bg-green-500 text-white border-0",
          type: "background",
        });
        clearDialog();
        setOpen(false);
      } else if ((new Date).valueOf() > Number(localStorage.getItem("invoice-exp-time"))) {
        toast({
          title: "Invoice was expired",
          type: "background",
          className: "bg-red-500 text-white border-0",
        });
        clearDialog()
      }
    }
  });

  useEffect(() => {
    setInvoice(localStorage.getItem("invoice"))
    setInterval(() => { if (localStorage.getItem("invoice")) checkInvoice(); }, POOLING_INTERVAL_TIME)
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
    onSuccess: data => {
      setInvoice(data.invoice);
      localStorage.setItem("invoice", data.invoice);
      localStorage.setItem("invoice-exp-time", ((new Date).valueOf() + INVOICE_EXP_TIME).toString());
    },
    onError: (error) => setError(error.message),
  });

  const formikbag = useFormik({
    initialValues: {
      amount: 1,
    },
    validate,
    onSubmit: async (values: Values, { resetForm }) => {
      await mutate(values);
      resetForm()
    },
  });

  const { handleSubmit } = formikbag;

  return (
    <Dialog open={open} onOpenChange={setOpen} >
      <DialogContent className="bg-white">
        <DialogHeader>
          <DialogTitle>{!invoice ? "Deposit" : "Invoice"} </DialogTitle>
          <DialogDescription>{!invoice ? "Add satoshis to your account." : "Use qr-code or copy/paste invoice to confirm payment"}</DialogDescription>
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
            <QRCode className="m-4" value={invoice} />
            <div className="flex flex-row w-[58%]">
              <div
                className="bg-[#3aafff] m-1 w-4/5 p-2 rounded-md flex justify-center text-white items-center hover:bg-[#80cfff]"
                onClick={async () => { await navigator.clipboard.writeText(invoice); }}>
                <Copy />
              </div>

              <div
                className="bg-red-500 m-1 w-1/5 p-2 rounded-md flex justify-center text-white items-center hover:bg-red-400"
                onClick={() => {
                  toast({
                    title: "Invoice was cancelled",
                    type: "background",
                    className: "bg-red-500 text-white border-0",
                  });
                  clearDialog()
                }}>
                <X />
              </div>
            </div>
          </div>
        }
      </DialogContent>
    </Dialog >
  );
};

export default DepositDialog;
