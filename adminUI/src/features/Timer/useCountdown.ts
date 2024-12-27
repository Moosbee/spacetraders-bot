import { useEffect, useState } from "react";

function useCountdown(time: Date) {
  const timeRemaining = time.getTime() - Date.now();
  const [rerender, setRerender] = useState(0);

  useEffect(() => {
    const timerInterval = setInterval(() => {
      // Update the component state with the new time remaining
      setRerender(rerender + 1);
    }, 1000);

    // Cleanup the interval when the component unmounts
    return () => clearInterval(timerInterval);
  }, [rerender]); // The empty dependency array ensures the effect runs only once on mount

  // Convert seconds to hours, minutes, and seconds
  let hours = Math.floor(timeRemaining / (1000 * 60 * 60));
  let minutes = Math.floor((timeRemaining % (1000 * 60 * 60)) / (1000 * 60));
  let seconds = Math.floor((timeRemaining % (1000 * 60)) / 1000);
  let milliseconds = Math.floor(timeRemaining % 1000);

  if (hours < 0) hours = hours + 1;
  if (minutes < 0) minutes = minutes + 1;
  if (seconds < 0) seconds = seconds + 1;
  if (milliseconds < 0) milliseconds = milliseconds + 1;

  return { hours, minutes, seconds, milliseconds };
}

export default useCountdown;
