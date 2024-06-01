import { ZodError, ZodIssue, ZodSchema } from 'zod';

const normalizeErrors = (errors: ZodIssue[]) => {
  const errorsNormalized = errors.map((error) => ({
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore
    field: error.validation,
    message: error.message,
  }));

  return errorsNormalized.reduce((acc, error) => {
    if (!acc[error.field]) {
      acc[error.field] = error;
    }

    return acc;
  }, {} as Record<string, { field: string; message: string }>);
};

type ValidationActionArgs = {
  request: Request;
  schema: ZodSchema;
};

export const validationAction = async ({
  request,
  schema,
}: ValidationActionArgs) => {
  const formData = Object.fromEntries(await request.formData());

  try {
    schema.parse(formData);

    return {
      formData,
      errors: [],
      success: true,
    };
  } catch (error) {
    if (error instanceof ZodError) {
      return {
        formData,
        errors: normalizeErrors(error.errors),
        success: false,
      };
    }

    return {
      formData,
      errors: [],
      success: true,
    };
  }
};
