import { FC, useState, useEffect } from 'react';
import { Container, TextField, Grid, FormControl, Typography, Button, Card, CardContent, Accordion, AccordionSummary, AccordionDetails, IconButton } from '@material-ui/core'
import Autocomplete from '@material-ui/lab/Autocomplete'
import { useForm, Control, Controller, useFieldArray } from 'react-hook-form'
import ExpandMoreIcon from '@material-ui/icons/ExpandMore'
import DeleteIcon from '@material-ui/icons/Delete'

interface Job {
  cronjob_data: CronJobData
  spec: JobSpec
}

interface CronJobData {
  name: string
  namespace: string
}

interface JobSpec {
  containers: PodContainer[]
}

interface PodContainer {
  name: string
  image: string
  command: string[]
  args: string[]
  env: EnvValue[]
}

interface EnvValue {
  name: string
  value: string
}

interface JobSpecForm {
  containers: ContainerForm[]
}

interface ContainerForm {
  name: string
  image: string
  command: StringForm[]
  args: StringForm[]
  env: EnvValue[]
}

interface StringForm {
  value: string
}

const newContainerForm = (): ContainerForm => {
  return {
    name: "",
    image: "",
    command: [],
    args: [],
    env: [],
  }
}

const stringToStringForm = (s: string[]): StringForm[] => {
  return s.map(s => ({ value: s }))
}

const containersToContainerForms = (containers: PodContainer[]): ContainerForm[] => {
  return containers.map(c => ({
    name: c.name,
    image: c.image,
    command: stringToStringForm(c.command),
    args: stringToStringForm(c.args),
    env: c.env,
  }))
}

const fetchJobs = async (namespace: string): Promise<Job[]> => {
  const url = namespace ? `/api/cronjob?namespace=${namespace}` : '/api/cronjob'
  const r = await fetch(url)
  const data = await r.json()

  if (!r.ok) {
    throw data['message']
  }
  return data
}

interface CommandsFormProps {
  control: Control<JobSpecForm>
  containerIndex: number
  fieldName: "command" | "args"
}

const CommandsForm: FC<CommandsFormProps> = ({ control, containerIndex, fieldName }) => {
  const { fields, append, remove } = useFieldArray({ control, name: `containers.${containerIndex}.${fieldName}` as const })

  return (
    <Grid container spacing={1}>
      <Typography>{fieldName}</Typography>
      {
        fields.map((command, index) => (
          <Grid item xs={12} key={command.id}>
            <Grid container>
              <Grid item xs={10}>
                <Controller
                  name={`containers.${containerIndex}.${fieldName}.${index}.value` as const}
                  control={control}
                  rules={{ required: true }}
                  render={({ field }) => (
                    <TextField
                      {...field}
                      defaultValue={command.value}
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
        <Button onClick={() => append({ value: "" })}>Add</Button>
      </Grid>
    </Grid>
  )
}

interface EnvFormProps {
  control: Control<JobSpecForm>
  containerIndex: number
}

const EnvForm: FC<EnvFormProps> = ({ control, containerIndex }) => {
  const { fields, append, remove } = useFieldArray({ control, name: `containers.${containerIndex}.env` as const })

  return (
    <Grid container spacing={1}>
      <Typography>env</Typography>
      {
        fields.map((envValue, index) => (
          <Grid item xs={12} key={envValue.id}>
            <Grid container>
              <Grid item xs={5}>
                <Controller
                  name={`containers.${containerIndex}.env.${index}.name` as const}
                  control={control}
                  rules={{ required: true }}
                  render={({ field }) => (
                    <TextField
                      {...field}
                      defaultValue={envValue.name}
                      variant="outlined"
                      fullWidth
                    />
                  )}
                />
              </Grid>
              <Grid item xs={5}>
                <Controller
                  name={`containers.${containerIndex}.env.${index}.value` as const}
                  control={control}
                  rules={{ required: true }}
                  render={({ field }) => (
                    <TextField
                      {...field}
                      defaultValue={envValue.value}
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
        <Button onClick={() => append({ value: "" })}>Add</Button>
      </Grid>
    </Grid>
  )
}

const App = () => {
  const [namespace, setNamespace] = useState<string>("")
  const [jobs, setJobs] = useState<Job[]>([])

  useEffect(() => {
    fetchJobs(namespace).then(j => {
      setJobs(j)
    }).catch(e => {
      console.error(e)
    })
  }, [])

  const { control, setValue } = useForm<JobSpecForm>({
    defaultValues: {
      containers: [],
    },
  });
  const { fields, append, remove } = useFieldArray({
    control,
    name: 'containers'
  })

  console.log(fields)
  return (
    <div className="App">
      <Container style={{ paddingTop: 10 }}>
        <Card>
          <CardContent>
            <Grid container spacing={2}>
              <Grid item xs={12}>
                <Typography variant="h5">Select template cronjob</Typography>
              </Grid>
              <Grid item xs={12}>
                <FormControl fullWidth>
                  <Autocomplete
                    options={Array.from(new Set(jobs.map(j => j.cronjob_data.namespace)))}
                    onChange={(_, n) => setNamespace(n || "")}
                    renderInput={(params) => (
                      <TextField
                        {...params}
                        label="namespace"
                        variant="outlined" />
                    )}
                  />
                </FormControl>
              </Grid>

              <Grid item xs={12}>
                <FormControl fullWidth>
                  <Autocomplete
                    options={jobs.filter(j => namespace === "" || j.cronjob_data.namespace === namespace)}
                    getOptionLabel={option => option.cronjob_data.name}
                    onChange={(_, t) => setValue("containers", containersToContainerForms(t?.spec.containers || []))}
                    fullWidth
                    renderInput={(params) => <TextField {...params} label="cronjob" variant="outlined" />}
                  />
                </FormControl>
              </Grid>
            </Grid>
          </CardContent>
        </Card>

        <Typography variant="h4">Containers</Typography>

        <form>
          {
            fields.map((job, index) => (
              <Accordion key={job.id}>
                <AccordionSummary
                  expandIcon={<ExpandMoreIcon />}
                  aria-controls={`container-${index}`}
                  id={`container-${index}`}
                >
                  <Typography>{job.name}</Typography>
                  <IconButton onClick={() => remove(index)} style={{marginLeft: 10, padding: 0}}>
                    <DeleteIcon fontSize="small" />
                  </IconButton>
                </AccordionSummary>
                <AccordionDetails>
                  <Grid container key={job.id} spacing={2}>
                    <Grid item xs={12}>
                      <Controller
                        name={`containers.${index}.name` as const}
                        control={control}
                        rules={{ required: true }}
                        defaultValue={job.name}
                        render={({ field: { onChange, onBlur, value, ref, name } }) => {
                          console.log(value, name)
                          return (
                            <TextField
                              label="name"
                              fullWidth
                              variant="outlined"
                              onChange={onChange}
                              onBlur={onBlur}
                              value={value}
                              inputRef={ref}
                            />
                          )
                        }}
                      />
                    </Grid>

                    <Grid item xs={12}>
                      <Controller
                        name={`containers.${index}.image` as const}
                        control={control}
                        rules={{ required: true }}
                        defaultValue={job.image}
                        render={({ field }) => (
                          <TextField
                            {...field}
                            label="image"
                            fullWidth
                            variant="outlined"
                          />
                        )}
                      />
                    </Grid>

                    <Grid item xs={12}>
                      <CommandsForm control={control} containerIndex={index} fieldName="command" />
                    </Grid>

                    <Grid item xs={12}>
                      <CommandsForm control={control} containerIndex={index} fieldName="args" />
                    </Grid>

                    <Grid item xs={12}>
                      <EnvForm control={control} containerIndex={index} />
                    </Grid>

                  </Grid>
                </AccordionDetails>
              </Accordion>
            ))
          }
          <Button onClick={() => append(newContainerForm())}>Add container</Button>
          <Button type="submit">Create Job</Button>
        </form>
      </Container>
    </div>
  );
}

export default App;
