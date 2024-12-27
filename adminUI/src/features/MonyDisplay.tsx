function MoneyDisplay({
  amount,
  ...props
}: { amount: number } & React.HTMLAttributes<HTMLSpanElement>) {
  return <span {...props}>{amount.toLocaleString()}$</span>;
}

export default MoneyDisplay;
