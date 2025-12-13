import styles from "./TextLoop.module.css";

type Props = {
  texts: string[];
  /** duration of one full cycle in seconds (default 6) */
  duration?: number;
  direction?: "left-to-right" | "right-to-left";
};

function TextLoop({ texts, duration = 6, direction = "right-to-left" }: Props) {
  if (!texts || texts.length === 0) return null;

  // Duplicate the list so the animation can scroll seamlessly
  const items = [...texts, ...texts];

  const cssVars = { ["--duration"]: `${duration}s` } as React.CSSProperties;

  return (
    <div
      className={styles.wrapper} // set CSS variable for the duration in seconds
      style={cssVars}
      aria-hidden={false}
    >
      <div
        className={`${styles.strip} ${
          direction === "right-to-left"
            ? styles.rightToLeft
            : styles.leftToRight
        }`}
      >
        {items.map((text, index) => (
          <div key={index} className={styles.item}>
            {text}
          </div>
        ))}
      </div>
      <span className={styles.srOnly} aria-hidden>
        {texts[0]}
      </span>
    </div>
  );
}

export default TextLoop;
