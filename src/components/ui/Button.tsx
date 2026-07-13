import { forwardRef, type ButtonHTMLAttributes } from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";

const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 rounded-xl text-sm font-semibold transition-colors disabled:pointer-events-none disabled:opacity-50",
  {
    variants: {
      variant: {
        primary: "bg-sage-400 px-4 py-2.5 text-sage-900 hover:bg-sage-300",
        secondary: "border border-line bg-white/5 px-4 py-2.5 text-ink hover:bg-white/10",
        ghost: "px-3 py-2 text-muted hover:bg-white/5 hover:text-ink",
        destructive: "bg-red-600 px-4 py-2.5 text-white hover:bg-red-500",
      },
    },
    defaultVariants: { variant: "primary" },
  },
);

type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & VariantProps<typeof buttonVariants>;

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, ...props }, ref) => (
    <button ref={ref} className={cn(buttonVariants({ variant }), className)} {...props} />
  ),
);

Button.displayName = "Button";
