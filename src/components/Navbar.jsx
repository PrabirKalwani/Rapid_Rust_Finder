import { Button } from "@/components/ui/button";
import { ChevronRight, ChevronLeft } from "lucide-react";
import { HomeIcon } from "@radix-ui/react-icons";
import { Input } from "@/components/ui/input";
import { MagnifyingGlassIcon } from "@radix-ui/react-icons";
import React from "react";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";
import { SidebarTrigger } from "./ui/sidebar";
import { Separator } from "./ui/separator";

export const Navbar = ({ text, path }) => {
  return (
    <header className="flex sticky top-0 bg-background h-16 shrink-0 items-center gap-2 border-b px-4">
      <SidebarTrigger className="-ml-1" />
      <Separator orientation="vertical" className="mr-2 h-4" />
      {path === null ? (
        <span className="text-lg">{text}</span>
      ) : (
        <Breadcrumb>
          <BreadcrumbList>
            {path
              .split("\\")
              .filter((folder) => folder)
              .map((folder, index, arr) => (
                <React.Fragment key={index}>
                  <BreadcrumbItem className="hidden md:block">
                    <BreadcrumbLink href="#" className="truncate">
                      {folder}
                    </BreadcrumbLink>
                  </BreadcrumbItem>
                  {index < arr.length - 1 && (
                    <BreadcrumbSeparator className="hidden md:block" />
                  )}
                </React.Fragment>
              ))}
          </BreadcrumbList>
        </Breadcrumb>
      )}
    </header>
  );
};
