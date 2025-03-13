function MoneyDisplay({
  amount,
  ...props
}: { amount: number } & React.HTMLAttributes<HTMLSpanElement>) {
  return <span {...props}>{(amount || 0).toLocaleString()}$</span>;
}

export default MoneyDisplay;
