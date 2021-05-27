import { useState, useEffect } from 'react';
import { Container, TextField, Grid, FormControl, Typography, Button, Card, CardContent, Accordion, AccordionSummary, AccordionDetails, IconButton } from '@material-ui/core'
import Autocomplete from '@material-ui/lab/Autocomplete'
import { useForm, Controller, useFieldArray } from 'react-hook-form'
import ExpandMoreIcon from '@material-ui/icons/ExpandMore'
import DeleteIcon from '@material-ui/icons/Delete'
import { CommandsForm, EnvForm } from './components'
import { Job, PodContainer, JobForm, ContainerForm, StringForm } from './models'

const newContainerForm = (): ContainerForm => {
  return {
    name: "",
    image: "",
    command: [],
    args: [],
    env: [],
  }
}

const stringsToStringForms = (s: string[]): StringForm[] => {
  return s.map(s => ({ value: s }))
}

const stringsFormTostrings = (s: StringForm[]): string[] => {
  return s.map(s => s.value)
}

const containersToContainerForms = (containers: PodContainer[]): ContainerForm[] => {
  return containers.map(c => ({
    name: c.name,
    image: c.image,
    command: stringsToStringForms(c.command),
    args: stringsToStringForms(c.args),
    env: c.env,
  }))
}

const containerFormsToContainers = (containerForms: ContainerForm[]): PodContainer[] => {
  return containerForms.map(c => ({
    name: c.name,
    image: c.image,
    command: stringsFormTostrings(c.command),
    args: stringsFormTostrings(c.args),
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

const createJob = async (job: Job): Promise<string> => {
  const url = "/api/job"
  const r = await fetch(url, {
    method: "POST",
    body: JSON.stringify(job),
    headers: new Headers({
      "Content-Type": "application/json",
    }),
  })
  const data = await r.json()

  if (!r.ok) {
    throw data['message']
  }
  return data['message']
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

  const { control, setValue, handleSubmit } = useForm<JobForm>({
    defaultValues: {
      cronjob_data: {
        name: "",
        namespace: "",
      },
      spec: {
        containers: [],
      },
    },
  });
  const { fields, append, remove } = useFieldArray({
    control,
    name: 'spec.containers'
  })

  const onSubmit = (data: JobForm) => {
    const request: Job = {
      cronjob_data: data.cronjob_data,
      spec: {
        containers: containerFormsToContainers(data.spec.containers),
      },
    }
    createJob(request).then(mes => {
      console.log(mes)
    }).catch(e => {
      console.error(e)
    })
  }

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
                    onChange={(_, t) => {
                      setValue("cronjob_data", t?.cronjob_data || { name: "", namespace: "" })
                      setValue("spec.containers", containersToContainerForms(t?.spec.containers || []))
                    }}
                    fullWidth
                    renderInput={(params) => <TextField {...params} label="cronjob" variant="outlined" />}
                  />
                </FormControl>
              </Grid>
            </Grid>
          </CardContent>
        </Card>

        <form onSubmit={handleSubmit(onSubmit)}>
          <Grid container spacing={2} style={{ marginTop: 10 }}>
            <Grid item xs={12}>
              <Typography variant="h4">Containers</Typography>
            </Grid>

            <Grid item xs={12}>
              {
                fields.map((job, index) => (
                  <Accordion key={job.id}>
                    <AccordionSummary
                      expandIcon={<ExpandMoreIcon />}
                      aria-controls={`container-${index}`}
                      id={`container-${index}`}
                    >
                      <Typography>{job.name}</Typography>
                      <IconButton onClick={() => remove(index)} style={{ marginLeft: 10, padding: 0 }}>
                        <DeleteIcon fontSize="small" />
                      </IconButton>
                    </AccordionSummary>
                    <AccordionDetails>
                      <Grid container key={job.id} spacing={2}>
                        <Grid item xs={12}>
                          <Controller
                            name={`spec.containers.${index}.name` as const}
                            control={control}
                            rules={{ required: true }}
                            defaultValue={job.name}
                            render={({ field: { onChange, onBlur, value, ref } }) => {
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
                            name={`spec.containers.${index}.image` as const}
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
            </Grid>

            <Grid item xs={12}>
              <Button onClick={() => append(newContainerForm())} variant="outlined" color="primary">Add container</Button>
              <Button type="submit" variant="contained" color="primary" style={{ marginLeft: 5 }}>Create Job</Button>
            </Grid>
          </Grid>
        </form>
      </Container>
    </div>
  );
}

export default App;
