import * as React from "react"
import { cn } from "@/lib/utils"

const ResizablePanelGroup = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement> & {
    direction?: "horizontal" | "vertical"
  }
>(({ className, direction = "horizontal", ...props }, ref) => (
  <div
    ref={ref}
    className={cn(
      "flex h-full w-full",
      direction === "horizontal" ? "flex-row" : "flex-col",
      className
    )}
    {...props}
  />
))
ResizablePanelGroup.displayName = "ResizablePanelGroup"

const ResizablePanel = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement> & {
    defaultSize?: number
  }
>(({ className, defaultSize, style, ...props }, ref) => (
  <div
    ref={ref}
    className={cn("flex-1", className)}
    style={{
      flex: defaultSize ? `0 0 ${defaultSize}%` : "1",
      ...style
    }}
    {...props}
  />
))
ResizablePanel.displayName = "ResizablePanel"

const ResizableHandle = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement> & {
    withHandle?: boolean
  }
>(({ className, withHandle, ...props }, ref) => (
  <div
    ref={ref}
    className={cn(
      "relative flex w-px items-center justify-center bg-border after:absolute after:inset-y-0 after:left-1/2 after:w-1 after:-translate-x-1/2 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring focus-visible:ring-offset-1 data-[panel-group-direction=vertical]:h-px data-[panel-group-direction=vertical]:w-full data-[panel-group-direction=vertical]:after:left-0 data-[panel-group-direction=vertical]:after:h-1 data-[panel-group-direction=vertical]:after:w-full data-[panel-group-direction=vertical]:after:-translate-y-1/2 data-[panel-group-direction=vertical]:after:translate-x-0 [&[data-panel-group-direction=vertical]>div]:rotate-90",
      className
    )}
    {...props}
  >
    {withHandle && (
      <div className="z-10 flex h-4 w-3 items-center justify-center rounded-sm border bg-border">
        <div className="h-2.5 w-[3px] rounded-full bg-border" />
      </div>
    )}
  </div>
))
ResizableHandle.displayName = "ResizableHandle"

export { ResizablePanelGroup, ResizablePanel, ResizableHandle }