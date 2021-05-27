import { FC } from 'react'
import { TextField, Grid, Typography, Button, IconButton } from '@material-ui/core'
import DeleteIcon from '@material-ui/icons/Delete'
import { Control, Controller, useFieldArray } from 'react-hook-form'
import { JobForm } from '../models'

interface CommandsFormProps {
  control: Control<JobForm>
  containerIndex: number
  fieldName: "command" | "args"
}

const CommandsForm: FC<CommandsFormProps> = ({ control, containerIndex, fieldName }) => {
  const { fields, append, remove } = useFieldArray({ control, name: `spec.containers.${containerIndex}.${fieldName}` as `spec.containers.0.command` })

  return (
    <Grid container spacing={1}>
      <Typography>{fieldName}</Typography>
      {
        fields.map((command, index) => (
          <Grid item xs={12} key={command.id}>
            <Grid container>
              <Grid item xs={10}>
                <Controller
                  name={`spec.containers.${containerIndex}.${fieldName}.${index}.value` as const}
                  control={control}
                  rules={{ required: true }}
                  defaultValue={command.value}
                  render={({ field }) => (
                    <TextField
                      {...field}
                      variant="outlined"
                      fullWidth
                    />
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
        ))
      }
      <Grid item xs={12}>
        <Button onClick={() => append({ value: "" })} variant="outlined" color="primary">Add</Button>
      </Grid>
    </Grid>
  )
}

export default CommandsForm
