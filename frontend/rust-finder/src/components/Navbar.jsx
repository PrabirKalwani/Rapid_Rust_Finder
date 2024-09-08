import React, { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import debounce from "lodash.debounce";

import { Button } from "@/components/ui/button";
import { ChevronRight, ChevronLeft } from "lucide-react";
import { HomeIcon } from "@radix-ui/react-icons";
import { Input } from "@/components/ui/input";
import { MagnifyingGlassIcon } from "@radix-ui/react-icons";

export const Navbar = () => {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  const fetchResults = async (query) => {
    setLoading(true);
    setError(null);
    try {
      const searchResults = await invoke("search_files", { query });
      setResults(searchResults);
    } catch (error) {
      console.error("Error searching files:", error);
      setError("Failed to search files.");
    } finally {
      setLoading(false);
    }
  };

  const debouncedFetchResults = useCallback(
    debounce((query) => fetchResults(query), 300),
    []
  );

  const handleChange = (e) => {
    const newQuery = e.target.value;
    setQuery(newQuery);
    if (newQuery.trim() === "") {
      setResults([]);
    } else {
      debouncedFetchResults(newQuery);
    }
  };
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
