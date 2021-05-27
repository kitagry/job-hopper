export interface Job {
  cronjob_data: CronJobData
  spec: JobSpec
}

export interface CronJobData {
  name: string
  namespace: string
}

export interface JobSpec {
  containers: PodContainer[]
}

export interface PodContainer {
  name: string
  image: string
  command: string[]
  args: string[]
  env: EnvVar[]
}

export interface EnvVar {
  name: string
  value: string
  value_from: unknown
}

export interface JobForm {
  cronjob_data: CronJobData
  spec: JobSpecForm
}

export interface JobSpecForm {
  containers: ContainerForm[]
}

export interface ContainerForm {
  name: string
  image: string
  command: StringForm[]
  args: StringForm[]
  env: EnvVar[]
}

export interface StringForm {
  value: string
}
