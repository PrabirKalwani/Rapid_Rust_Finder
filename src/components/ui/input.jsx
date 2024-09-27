import * as React from "react";

import { cn } from "@/lib/utils";

const Input = React.forwardRef(({ className, type, icon,...props }, ref) => {
  return (
    <div
      className={cn(
        "flex h-9 w-full rounded-md border border-input bg-transparent text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50",
        className
      )}
    >
      {icon && <div className="flex items-center justify-center basis-1/15 p-3">
        {icon}
      </div>}
      <input className="basis-14/15 bg-transparent h-full border-none focus:outline-none" type={type} ref={ref} {...props} />
    </div>
  );
});
Input.displayName = "Input";

export { Input };