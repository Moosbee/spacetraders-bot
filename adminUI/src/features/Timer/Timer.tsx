import useCountdown from "./useCountdown";

function Timer({ time }: { time: string }) {
  const { hours, minutes, seconds } = useCountdown(new Date(time));
  return (
    <>
      {hours.toString().padStart(2, "0")}:{minutes.toString().padStart(2, "0")}:
      {seconds.toString().padStart(2, "0")}
    </>
  );
}

export default Timer;
