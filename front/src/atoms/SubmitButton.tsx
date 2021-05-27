import { FC } from 'react';
import { Button, CircularProgress } from '@material-ui/core';
import { makeStyles } from '@material-ui/core/styles';

const useStyles = makeStyles((theme) => ({
  root: {
    display: 'flex',
    alignItems: 'center',
  },
  wrapper: {
    margin: theme.spacing(1),
    position: 'relative',
  },
  buttonProgress: {
    position: 'absolute',
    top: '50%',
    left: '50%',
    marginTop: -12,
    marginLeft: -12,
  },
}));

interface Props {
  isLoading: boolean;
}

export const SubmitButton: FC<Props> = ({ isLoading, children }) => {
  const classes = useStyles();

  return (
    <div className={classes.wrapper}>
      <Button
        type="submit"
        variant="contained"
        color="primary"
        disabled={isLoading}
      >
        {children}
      </Button>
      {isLoading && (
        <CircularProgress size={24} className={classes.buttonProgress} />
      )}
    </div>
  );
};
