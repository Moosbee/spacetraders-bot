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
  const hours = Math.floor(timeRemaining / (1000 * 60 * 60));
  const minutes = Math.floor((timeRemaining % (1000 * 60 * 60)) / (1000 * 60));
  const seconds = Math.floor((timeRemaining % (1000 * 60)) / 1000);
  const milliseconds = Math.floor(timeRemaining % 1000);

  return { hours, minutes, seconds, milliseconds };
}

export default useCountdown;
