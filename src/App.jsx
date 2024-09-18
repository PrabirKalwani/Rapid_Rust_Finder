import React, { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import debounce from "lodash.debounce";
import "./App.css";
import { ThemeProvider } from "@/components/theme-provider";
import { Navbar } from "@/components/Navbar";
import { ViewPage } from "@/components/ViewPage";

function App() {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  const fetchResults = async (query) => {
    setLoading(true);
    setError(null);
    try {
      const searchResults = await invoke("search_files", { query });
      console.log(typeof(searchResults))
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

  const openFile = (filePath) => {
    if (filePath) {
      window.open(filePath, "_blank");
    } else {
      console.warn("No file path provided for opening.");
    }
  };

  return (
    <ThemeProvider defaultTheme="dark" storageKey="vite-ui-theme">
      <div className="App">
        <Navbar query={query} handleChange={handleChange}></Navbar>
        <ViewPage results={results} loading={loading} error={error} query={query} openFile={openFile}></ViewPage>
      </div>
    </ThemeProvider>
  );
}

export default App;
