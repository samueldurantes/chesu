import * as React from "react"
import {useField} from 'formik';

import { cn } from "@/utils/cn";

export interface InputProps
  extends React.InputHTMLAttributes<HTMLInputElement> {
    name: string;
}

const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ className, type, ...props }, ref) => {
    const [field, meta] = useField(props.name);

    const hasAnErrorAndHasBeenTouched = !!meta.error && !!meta.touched;

    return (
      <div className="flex flex-col gap-2">
        <input
          type={type}
          className={cn(
            "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50",
            className
          )}
          ref={ref}
          {...props}
          {...field}
        />
        {hasAnErrorAndHasBeenTouched && (
          <span className="text-red-500 text-sm">
            {meta.error}
          </span>
        )}
      </div>
    )
  }
)
Input.displayName = "Input"

export { Input }
