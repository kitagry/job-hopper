import { FC } from 'react';
import {
  TextField,
  Grid,
  Typography,
  Button,
  IconButton,
} from '@material-ui/core';
import DeleteIcon from '@material-ui/icons/Delete';
import { Control, Controller, useFieldArray } from 'react-hook-form';
import { JobForm } from '../models';

interface EnvFormProps {
  control: Control<JobForm>;
  containerIndex: number;
}

const EnvForm: FC<EnvFormProps> = ({ control, containerIndex }) => {
  const { fields, append, remove } = useFieldArray({
    control,
    name: `spec.containers.${containerIndex}.env` as `spec.containers.0.env`,
  });

  return (
    <Grid container spacing={1}>
      <Typography>env</Typography>
      {fields.map((envValue, index) => (
        <Grid item xs={12} key={envValue.id}>
          <Grid container>
            <Grid item xs={5}>
              <Controller
                name={
                  `spec.containers.${containerIndex}.env.${index}.name` as const
                }
                control={control}
                defaultValue={envValue.name}
                rules={{ required: true }}
                render={({ field }) => (
                  <TextField {...field} variant="outlined" fullWidth />
                )}
              />
            </Grid>
            <Grid item xs={5}>
              <Controller
                name={
                  `spec.containers.${containerIndex}.env.${index}.value` as const
                }
                control={control}
                defaultValue={envValue.value}
                render={({ field }) => (
                  <TextField {...field} variant="outlined" fullWidth />
                )}
              />
            </Grid>
            <Grid item xs={2}>
              <IconButton onClick={() => remove(index)}>
                <DeleteIcon />
              </IconButton>
            </Grid>
          </Grid>
        </Grid>
      ))}
      <Grid item xs={12}>
        <Button
          onClick={() => append({ value: '' })}
          variant="outlined"
          color="primary"
        >
          Add
        </Button>
      </Grid>
    </Grid>
  );
};

export default EnvForm;
