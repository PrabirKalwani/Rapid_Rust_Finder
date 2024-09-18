import { Button } from "@/components/ui/button";
import { ChevronRight, ChevronLeft } from "lucide-react";
import { HomeIcon } from "@radix-ui/react-icons";
import { Input } from "@/components/ui/input";
import { MagnifyingGlassIcon } from "@radix-ui/react-icons";

export const Navbar = ({query, handleChange}) => {

  return (
    <section className="Navbar grid grid-cols-9 gap-2 p-2">
      <span className="nav-buttons flex flex-row gap-2 col-span-2">
        <Button variant="ghost" size="icon">
          <ChevronLeft className="h-4 w-4" />
        </Button>
        <Button variant="ghost" size="icon">
          <ChevronRight className="h-4 w-4" />
        </Button>
      </span>
      <Input className="col-span-5" icon={<HomeIcon />}></Input>
      <Input
        className="col-span-2"
        icon={<MagnifyingGlassIcon />}
        value={query}
        onChange={handleChange}
        placeholder="Search files..."
      ></Input>
    </section>
  );
};
